use std::{collections::{HashMap, HashSet}, env, fs, path::PathBuf, process::Command};

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Ident(String), Num(i64), LParen, RParen, LBrace, RBrace, Semi, Comma,
    Plus, Minus, Star, Slash, Percent, Eq, EqEq, Ne, Lt, Le, Gt, Ge, Bang, AndAnd, OrOr, Arrow, Eof
}
#[derive(Clone, Debug)]
struct Token { kind: Tok, line: usize, col: usize }

struct Lexer { chars: Vec<char>, pos: usize, line: usize, col: usize }
impl Lexer {
    fn new(s:&str)->Self { Self{chars:s.chars().collect(),pos:0,line:1,col:1} }
    fn peek(&self)->char { *self.chars.get(self.pos).unwrap_or(&'\0') }
    fn bump(&mut self)->char { let c=self.peek(); if c!='\0' {self.pos+=1;if c=='\n'{self.line+=1;self.col=1}else{self.col+=1}} c }
    fn tokens(mut self)->Result<Vec<Token>,String>{
        let mut out=Vec::new();
        loop {
            while self.peek().is_whitespace(){self.bump();}
            let (line,col,c)=(self.line,self.col,self.peek());
            if c=='\0'{out.push(Token{kind:Tok::Eof,line,col});break;}
            let kind=if c.is_ascii_alphabetic()||c=='_'{
                let mut s=String::new();while self.peek().is_ascii_alphanumeric()||self.peek()=='_'{s.push(self.bump());}Tok::Ident(s)
            }else if c.is_ascii_digit(){
                let mut s=String::new();while self.peek().is_ascii_digit(){s.push(self.bump());}
                Tok::Num(s.parse().map_err(|_|format!("{line}:{col}: integer literal out of range"))?)
            }else{
                self.bump();
                match c {
                    '('=>Tok::LParen,')'=>Tok::RParen,'{'=>Tok::LBrace,'}'=>Tok::RBrace,';'=>Tok::Semi,','=>Tok::Comma,
                    '+'=>Tok::Plus,'-'=>if self.peek()=='>'{self.bump();Tok::Arrow}else{Tok::Minus},
                    '*'=>Tok::Star,'/'=>Tok::Slash,'%'=>Tok::Percent,
                    '='=>if self.peek()=='='{self.bump();Tok::EqEq}else{Tok::Eq},
                    '!'=>if self.peek()=='='{self.bump();Tok::Ne}else{Tok::Bang},
                    '<'=>if self.peek()=='='{self.bump();Tok::Le}else{Tok::Lt},
                    '>'=>if self.peek()=='='{self.bump();Tok::Ge}else{Tok::Gt},
                    '&'=>if self.peek()=='&'{self.bump();Tok::AndAnd}else{return Err(format!("{line}:{col}: expected '&'"))},
                    '|'=>if self.peek()=='|'{self.bump();Tok::OrOr}else{return Err(format!("{line}:{col}: expected '|'"))},
                    _=>return Err(format!("{line}:{col}: unexpected character '{c}'")),
                }
            };
            out.push(Token{kind,line,col});
        } Ok(out)
    }
}

#[derive(Clone,Debug)]
enum Expr { Num(i64), Var(String), Unary(Tok,Box<Expr>), Binary(Tok,Box<Expr>,Box<Expr>) }
#[derive(Clone,Debug)]
enum Stmt {
    Let(String,Expr), Assign(String,Expr), Print(Expr), Return(Expr), Break, Continue,
    Block(Vec<Stmt>), If(Expr,Box<Stmt>,Option<Box<Stmt>>),
    While(Expr,Box<Stmt>), For(Option<Box<Stmt>>,Option<Expr>,Option<Box<Stmt>>,Box<Stmt>),
}
struct Parser{ts:Vec<Token>,p:usize}
impl Parser {
    fn cur(&self)->&Token{&self.ts[self.p.min(self.ts.len()-1)]}
    fn take(&mut self)->Token{let t=self.cur().clone();if t.kind!=Tok::Eof{self.p+=1;}t}
    fn err(&self,m:&str)->String{format!("{}:{}: {m}",self.cur().line,self.cur().col)}
    fn eat(&mut self,k:Tok)->bool{if self.cur().kind==k{self.p+=1;true}else{false}}
    fn expect(&mut self,k:Tok,m:&str)->Result<(),String>{if self.eat(k){Ok(())}else{Err(self.err(m))}}
    fn ident(&mut self,m:&str)->Result<String,String>{match self.take().kind{Tok::Ident(s)=>Ok(s),_=>Err(self.err(m))}}
    fn program(mut self)->Result<Vec<Stmt>,String>{
        if self.ident("expected fn")?!="fn"{return Err(self.err("expected fn"));}
        if self.ident("expected main")?!="main"{return Err(self.err("only main is supported"));}
        self.expect(Tok::LParen,"expected '('")?;self.expect(Tok::RParen,"expected ')'")?;
        self.expect(Tok::Arrow,"expected '->'")?;
        if self.ident("expected i32")?!="i32"{return Err(self.err("main must return i32"));}
        let body=self.block()?;
        if self.cur().kind!=Tok::Eof{return Err(self.err("unexpected tokens after main"));}
        if !body.iter().any(|s|matches!(s,Stmt::Return(_))){return Err(self.err("main must contain return"));}
        Ok(body)
    }
    fn block(&mut self)->Result<Vec<Stmt>,String>{
        self.expect(Tok::LBrace,"expected '{'")?;let mut v=Vec::new();
        while self.cur().kind!=Tok::RBrace&&self.cur().kind!=Tok::Eof{v.push(self.stmt()?);}
        self.expect(Tok::RBrace,"expected '}'")?;Ok(v)
    }
    fn simple(&mut self,semi:bool)->Result<Stmt,String>{
        let w=self.ident("expected statement")?;
        let s=match w.as_str(){
            "i64"=>{let n=self.ident("expected variable name")?;self.expect(Tok::Eq,"expected '='")?;Stmt::Let(n,self.expr(0)?)},
            "print"=>{self.expect(Tok::LParen,"expected '('")?;let e=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;Stmt::Print(e)},
            "return"=>Stmt::Return(self.expr(0)?),
            "break"=>Stmt::Break,
            "continue"=>Stmt::Continue,
            _=>{if self.eat(Tok::Eq){Stmt::Assign(w,self.expr(0)?)}else{return Err(self.err(&format!("unknown statement '{w}'")))}}
        };
        if semi{self.expect(Tok::Semi,"expected ';'")?;}Ok(s)
    }
    fn stmt(&mut self)->Result<Stmt,String>{
        if self.cur().kind==Tok::LBrace{return Ok(Stmt::Block(self.block()?));}
        if let Tok::Ident(ref s)=self.cur().kind {
            match s.as_str(){
                "if"=>{self.take();self.expect(Tok::LParen,"expected '(' after if")?;let c=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;let a=Box::new(self.stmt()?);let b=if let Tok::Ident(ref x)=self.cur().kind{if x=="else"{self.take();Some(Box::new(self.stmt()?))}else{None}}else{None};return Ok(Stmt::If(c,a,b));},
                "while"=>{self.take();self.expect(Tok::LParen,"expected '(' after while")?;let c=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;return Ok(Stmt::While(c,Box::new(self.stmt()?)));},
                "for"=>{self.take();self.expect(Tok::LParen,"expected '(' after for")?;
                    let init=if self.eat(Tok::Semi){None}else{let x=Box::new(self.simple(false)?);self.expect(Tok::Semi,"expected ';' after for initializer")?;Some(x)};
                    let cond=if self.eat(Tok::Semi){None}else{let x=self.expr(0)?;self.expect(Tok::Semi,"expected ';' after for condition")?;Some(x)};
                    let step=if self.cur().kind==Tok::RParen{None}else{Some(Box::new(self.simple(false)?))};
                    self.expect(Tok::RParen,"expected ')' after for header")?;return Ok(Stmt::For(init,cond,step,Box::new(self.stmt()?)));},
                _=>{}
            }
        }
        self.simple(true)
    }
    fn expr(&mut self,min:u8)->Result<Expr,String>{
        let mut lhs=match self.take().kind{
            Tok::Num(n)=>Expr::Num(n),Tok::Ident(s)=>match s.as_str(){"true"=>Expr::Num(1),"false"=>Expr::Num(0),_=>Expr::Var(s)},
            Tok::Minus=>Expr::Unary(Tok::Minus,Box::new(self.expr(8)?)),
            Tok::Plus=>Expr::Unary(Tok::Plus,Box::new(self.expr(8)?)),
            Tok::Bang=>Expr::Unary(Tok::Bang,Box::new(self.expr(8)?)),
            Tok::LParen=>{let e=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;e},
            _=>return Err(self.err("expected expression")),
        };
        loop{
            let (op,p)=match self.cur().kind{
                Tok::OrOr=>(Tok::OrOr,1),Tok::AndAnd=>(Tok::AndAnd,2),
                Tok::EqEq=>(Tok::EqEq,3),Tok::Ne=>(Tok::Ne,3),
                Tok::Lt=>(Tok::Lt,4),Tok::Le=>(Tok::Le,4),Tok::Gt=>(Tok::Gt,4),Tok::Ge=>(Tok::Ge,4),
                Tok::Plus=>(Tok::Plus,5),Tok::Minus=>(Tok::Minus,5),
                Tok::Star=>(Tok::Star,6),Tok::Slash=>(Tok::Slash,6),Tok::Percent=>(Tok::Percent,6),_=>break
            };
            if p<min{break;}self.take();let rhs=self.expr(p+1)?;lhs=Expr::Binary(op,Box::new(lhs),Box::new(rhs));
        }Ok(lhs)
    }
}

struct Gen{asm:String,vars:HashMap<String,i32>,initialized:HashSet<String>,slots:i32,label:i32,breaks:Vec<String>,continues:Vec<String>}
impl Gen{
    fn line(&mut self,s:&str){self.asm.push_str(s);self.asm.push('\n');}
    fn fresh(&mut self,p:&str)->String{let n=self.label;self.label+=1;format!(".Lara_{p}_{n}")}
    fn collect(&mut self,ss:&[Stmt])->Result<(),String>{
        for s in ss{match s{
            Stmt::Let(n,_)=>{if self.vars.contains_key(n){return Err(format!("duplicate variable '{n}'"));}self.slots+=1;self.vars.insert(n.clone(),self.slots*8);},
            Stmt::Block(b)=>self.collect(b)?,
            Stmt::If(_,a,b)=>{self.collect(std::slice::from_ref(a.as_ref()))?;if let Some(x)=b{self.collect(std::slice::from_ref(x.as_ref()))?;}},
            Stmt::While(_,b)=>self.collect(std::slice::from_ref(b.as_ref()))?,
            Stmt::For(i,_,_,b)=>{if let Some(x)=i{self.collect(std::slice::from_ref(x.as_ref()))?;}self.collect(std::slice::from_ref(b.as_ref()))?;},
            _=>{}
        }}Ok(())
    }
    fn expr(&mut self,e:&Expr)->Result<(),String>{
        match e{
            Expr::Num(n)=>self.line(&format!("    mov rax, {n}")),
            Expr::Var(n)=>{if !self.initialized.contains(n){return Err(format!("use of undeclared or uninitialized variable '{n}'"));}let o=*self.vars.get(n).ok_or_else(||format!("unknown variable '{n}'"))?;self.line(&format!("    mov rax, QWORD PTR [rbp-{o}]"));},
            Expr::Unary(op,x)=>{self.expr(x)?;match op{Tok::Minus=>self.line("    neg rax"),Tok::Bang=>{self.line("    test rax, rax");self.line("    sete al");self.line("    movzx rax, al");},_=>{}}},
            Expr::Binary(op,a,b)=>{
                self.expr(a)?;self.line("    push rax");self.expr(b)?;self.line("    mov rcx, rax");self.line("    pop rax");
                match op{
                    Tok::Plus=>self.line("    add rax, rcx"),Tok::Minus=>self.line("    sub rax, rcx"),Tok::Star=>self.line("    imul rax, rcx"),
                    Tok::Slash=>{self.line("    cqo");self.line("    idiv rcx");},Tok::Percent=>{self.line("    cqo");self.line("    idiv rcx");self.line("    mov rax, rdx");},
                    Tok::EqEq|Tok::Ne|Tok::Lt|Tok::Le|Tok::Gt|Tok::Ge=>{self.line("    cmp rax, rcx");let ins=match op{Tok::EqEq=>"sete",Tok::Ne=>"setne",Tok::Lt=>"setl",Tok::Le=>"setle",Tok::Gt=>"setg",_=>"setge"};self.line(&format!("    {ins} al"));self.line("    movzx rax, al");},
                    Tok::AndAnd|Tok::OrOr=>{self.line("    test rax, rax");self.line("    setne al");self.line("    movzx rax, al");self.line("    test rcx, rcx");self.line("    setne cl");self.line("    movzx rcx, cl");if *op==Tok::AndAnd{self.line("    and rax, rcx")}else{self.line("    or rax, rcx")}},
                    _=>{}
                }
            }
        }Ok(())
    }
    fn stmt(&mut self,s:&Stmt)->Result<(),String>{
        match s{
            Stmt::Let(n,e)=>{self.expr(e)?;let o=self.vars[n];self.line(&format!("    mov QWORD PTR [rbp-{o}], rax"));self.initialized.insert(n.clone());},
            Stmt::Assign(n,e)=>{if !self.initialized.contains(n){return Err(format!("assignment to undeclared or uninitialized variable '{n}'"));}self.expr(e)?;let o=self.vars[n];self.line(&format!("    mov QWORD PTR [rbp-{o}], rax"));},
            Stmt::Print(e)=>{self.expr(e)?;self.line("    mov rsi, rax");self.line("    lea rdi, [rip + .Lara_fmt]");self.line("    xor eax, eax");self.line("    call printf@PLT");},
            Stmt::Return(e)=>{self.expr(e)?;self.line("    leave");self.line("    ret");},
            Stmt::Break=>{let l=self.breaks.last().ok_or_else(||"break used outside a loop".to_string())?.clone();self.line(&format!("    jmp {l}"));},
            Stmt::Continue=>{let l=self.continues.last().ok_or_else(||"continue used outside a loop".to_string())?.clone();self.line(&format!("    jmp {l}"));},
            Stmt::Block(b)=>for x in b{self.stmt(x)?;},
            Stmt::If(c,a,b)=>{let el=self.fresh("else");let end=self.fresh("ifend");self.expr(c)?;self.line("    test rax, rax");self.line(&format!("    je {el}"));let before=self.initialized.clone();self.stmt(a)?;let then_init=self.initialized.clone();if b.is_some(){self.line(&format!("    jmp {end}"));}self.line(&format!("{el}:"));self.initialized=before.clone();if let Some(x)=b{self.stmt(x)?;}let else_init=self.initialized.clone();self.initialized=if b.is_some(){then_init.intersection(&else_init).cloned().collect()}else{before};if b.is_some(){self.line(&format!("{end}:"));}},
            Stmt::While(c,b)=>{let top=self.fresh("while");let end=self.fresh("wend");let before=self.initialized.clone();self.line(&format!("{top}:"));self.expr(c)?;self.line("    test rax, rax");self.line(&format!("    je {end}"));self.breaks.push(end.clone());self.continues.push(top.clone());self.stmt(b)?;self.breaks.pop();self.continues.pop();self.line(&format!("    jmp {top}"));self.line(&format!("{end}:"));self.initialized=before;},
            Stmt::For(i,c,step,b)=>{if let Some(x)=i{self.stmt(x)?;}let top=self.fresh("for");let inc=self.fresh("finc");let end=self.fresh("fend");self.line(&format!("{top}:"));if let Some(x)=c{self.expr(x)?;self.line("    test rax, rax");self.line(&format!("    je {end}"));}self.breaks.push(end.clone());self.continues.push(inc.clone());self.stmt(b)?;self.breaks.pop();self.continues.pop();self.line(&format!("{inc}:"));if let Some(x)=step{self.stmt(x)?;}self.line(&format!("    jmp {top}"));self.line(&format!("{end}:"));},
        }Ok(())
    }
    fn generate(ss:&[Stmt])->Result<String,String>{
        let mut g=Self{asm:String::new(),vars:HashMap::new(),initialized:HashSet::new(),slots:0,label:0,breaks:Vec::new(),continues:Vec::new()};
        g.collect(ss)?;g.line(".intel_syntax noprefix");g.line(".text");g.line(".globl main");g.line(".type main, @function");g.line("main:");g.line("    push rbp");g.line("    mov rbp, rsp");
        let bytes=((g.slots*8+15)/16)*16;if bytes>0{g.line(&format!("    sub rsp, {bytes}"));}
        for s in ss{g.stmt(s)?;}
        g.line(".size main, .-main");g.line(".section .rodata");g.line(".Lara_fmt:");g.line("    .string \"%ld\\n\"");g.line(".section .note.GNU-stack,\"\",@progbits");Ok(g.asm)
    }
}

fn main(){if let Err(e)=run(){eprintln!("ara: error: {e}");std::process::exit(1);}}
fn run()->Result<(),String>{
    let args:Vec<String>=env::args().skip(1).collect();
    if args.len()<2||args[0]!="build"{return Err("usage: ara build <file.ara> [-o executable] [--emit-asm] [-O0]".into());}
    let input=PathBuf::from(&args[1]);let mut output=PathBuf::from(input.file_stem().unwrap_or_default());let mut emit=false;let mut i=2;
    while i<args.len(){match args[i].as_str(){"-o"=>{i+=1;if i>=args.len(){return Err("missing path after -o".into());}output=PathBuf::from(&args[i]);},"--emit-asm"=>emit=true,"-O0"=>{},"-O2"|"-O3"=>return Err("optimization levels above -O0 are not implemented".into()),"--target"=>return Err("only x86-64 Linux is implemented".into()),x=>return Err(format!("unknown option '{x}'"))}i+=1;}
    let src=fs::read_to_string(&input).map_err(|e|format!("cannot read {}: {e}",input.display()))?;
    let ts=Lexer::new(&src).tokens()?;let ast=Parser{ts,p:0}.program()?;let asm=Gen::generate(&ast)?;
    let ap=PathBuf::from(format!("{}.s",output.display()));fs::write(&ap,&asm).map_err(|e|format!("cannot write assembly: {e}"))?;
    if emit{println!("wrote {}",ap.display());return Ok(());}
    let st=Command::new("cc").arg("-no-pie").arg(&ap).arg("-o").arg(&output).status().map_err(|_|"could not start cc; install build-essential".to_string())?;
    if !st.success(){return Err("native linking failed".into());}println!("built {}",output.display());Ok(())
}
