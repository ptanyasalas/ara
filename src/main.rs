use std::{env, fs, path::{Path, PathBuf}, process::Command};

#[derive(Clone, Debug, PartialEq)]
enum Tok { Ident(String), Num(i64), LParen, RParen, LBrace, RBrace, Semi, Comma, Plus, Minus, Star, Slash, Percent, Eq, Arrow, Eof }

#[derive(Clone, Debug)]
struct Token { kind: Tok, line: usize, col: usize }

struct Lexer { chars: Vec<char>, pos: usize, line: usize, col: usize }
impl Lexer {
    fn new(s: &str) -> Self { Self { chars:s.chars().collect(), pos:0, line:1, col:1 } }
    fn peek(&self) -> char { *self.chars.get(self.pos).unwrap_or(&'\0') }
    fn bump(&mut self) -> char { let c=self.peek(); if c!='\0' { self.pos+=1; if c=='\n' {self.line+=1;self.col=1;} else {self.col+=1;} } c }
    fn tokens(mut self) -> Result<Vec<Token>,String> {
        let mut out=Vec::new();
        loop {
            while self.peek().is_whitespace() { self.bump(); }
            let line=self.line; let col=self.col; let c=self.peek();
            if c=='\0' { out.push(Token{kind:Tok::Eof,line,col}); break; }
            let kind=if c.is_ascii_alphabetic() || c=='_' {
                let mut s=String::new(); while self.peek().is_ascii_alphanumeric() || self.peek()=='_' {s.push(self.bump());} Tok::Ident(s)
            } else if c.is_ascii_digit() {
                let mut s=String::new(); while self.peek().is_ascii_digit() {s.push(self.bump());}
                Tok::Num(s.parse().map_err(|_|format!("{line}:{col}: integer literal out of range"))?)
            } else {
                self.bump();
                match c {
                    '('=>Tok::LParen, ')'=>Tok::RParen, '{'=>Tok::LBrace, '}'=>Tok::RBrace,
                    ';'=>Tok::Semi, ','=>Tok::Comma, '+'=>Tok::Plus, '-'=>if self.peek()=='>' {self.bump();Tok::Arrow}else{Tok::Minus},
                    '*'=>Tok::Star, '/'=>Tok::Slash, '%'=>Tok::Percent, '='=>Tok::Eq,
                    _=>return Err(format!("{line}:{col}: unexpected character '{c}'")),
                }
            };
            out.push(Token{kind,line,col});
        }
        Ok(out)
    }
}

#[derive(Clone, Debug)]
enum Expr { Num(i64), Var(String), Unary(char,Box<Expr>), Bin(char,Box<Expr>,Box<Expr>) }
#[derive(Clone, Debug)]
enum Stmt { Let(String,Expr), Assign(String,Expr), Print(Expr), Return(Expr) }
struct Parser { ts:Vec<Token>, p:usize }
impl Parser {
    fn cur(&self)->&Token { &self.ts[self.p] }
    fn take(&mut self)->Token {let t=self.ts[self.p].clone();self.p+=1;t}
    fn err(&self,msg:&str)->String {format!("{}:{}: {msg}",self.cur().line,self.cur().col)}
    fn ident(&mut self, expected:&str)->Result<String,String> { match self.take().kind {Tok::Ident(s)=>Ok(s),_=>Err(self.err(&format!("expected {expected}")))}}
    fn eat(&mut self,k:Tok)->bool {if self.cur().kind==k {self.p+=1;true}else{false}}
    fn expect(&mut self,k:Tok,msg:&str)->Result<(),String>{if self.eat(k){Ok(())}else{Err(self.err(msg))}}
    fn program(mut self)->Result<Vec<Stmt>,String>{
        if self.ident("fn")? != "fn" {return Err(self.err("expected fn"));}
        if self.ident("main")? != "main" {return Err(self.err("only main is supported in this compiler prototype"));}
        self.expect(Tok::LParen,"expected '('")?; self.expect(Tok::RParen,"expected ')'")?;
        self.expect(Tok::Arrow,"expected '->'")?;
        if self.ident("i32")? != "i32" {return Err(self.err("main must return i32"));}
        self.expect(Tok::LBrace,"expected '{'")?;
        let mut ss=Vec::new(); let mut returned=false;
        while self.cur().kind!=Tok::RBrace && self.cur().kind!=Tok::Eof {
            if returned {return Err(self.err("no statements are allowed after return"));}
            let word=self.ident("a statement")?;
            let st=match word.as_str() {
                "i64" => {let n=self.ident("variable name")?; self.expect(Tok::Eq,"expected '='")?;let e=self.expr(0)?;self.expect(Tok::Semi,"expected ';'")?;Stmt::Let(n,e)}
                "print" => {self.expect(Tok::LParen,"expected '(' after print")?;let e=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;self.expect(Tok::Semi,"expected ';'")?;Stmt::Print(e)}
                "return" => {let e=self.expr(0)?;self.expect(Tok::Semi,"expected ';'")?;returned=true;Stmt::Return(e)}
                _ => { if self.eat(Tok::Eq) {let e=self.expr(0)?;self.expect(Tok::Semi,"expected ';'")?;Stmt::Assign(word,e)} else {return Err(self.err(&format!("unknown statement '{word}'")));} }
            };
            ss.push(st);
        }
        self.expect(Tok::RBrace,"expected '}'")?;
        if self.cur().kind!=Tok::Eof {return Err(self.err("unexpected tokens after main"));}
        if !returned {return Err(self.err("main must end with return"));}
        Ok(ss)
    }
    fn expr(&mut self,min:u8)->Result<Expr,String>{
        let mut lhs=match self.take().kind {
            Tok::Num(n)=>Expr::Num(n),
            Tok::Ident(s)=>Expr::Var(s),
            Tok::Minus=>Expr::Unary('-',Box::new(self.expr(3)?)),
            Tok::Plus=>Expr::Unary('+',Box::new(self.expr(3)?)),
            Tok::LParen=>{let e=self.expr(0)?;self.expect(Tok::RParen,"expected ')'")?;e},
            _=>return Err(self.err("expected an expression")),
        };
        loop {
            let (op,prec)=match self.cur().kind {Tok::Plus=>( '+',1),Tok::Minus=>('-',1),Tok::Star=>('*',2),Tok::Slash=>('/',2),Tok::Percent=>('%',2),_=>break};
            if prec<min {break;} self.take(); let rhs=self.expr(prec+1)?; lhs=Expr::Bin(op,Box::new(lhs),Box::new(rhs));
        }
        Ok(lhs)
    }
}

struct Gen { asm:String, vars:std::collections::HashMap<String,i32>, slots:i32, label:u32 }
impl Gen {
    fn line(&mut self,s:&str){self.asm.push_str(s);self.asm.push('\n');}
    fn expr(&mut self,e:&Expr)->Result<(),String>{
        match e {
            Expr::Num(n)=>self.line(&format!("    mov rax, {n}")),
            Expr::Var(n)=>{let off=*self.vars.get(n).ok_or_else(||format!("unknown variable '{n}'"))?;self.line(&format!("    mov rax, QWORD PTR [rbp-{off}]"));},
            Expr::Unary(op,x)=>{self.expr(x)?;if *op=='-' {self.line("    neg rax");}},
            Expr::Bin(op,a,b)=>{
                self.expr(a)?; self.line("    push rax"); self.expr(b)?; self.line("    mov rcx, rax"); self.line("    pop rax");
                match op {'+'=>self.line("    add rax, rcx"),'-'=>self.line("    sub rax, rcx"),'*'=>self.line("    imul rax, rcx"),'/'=>{self.line("    cqo");self.line("    idiv rcx");},'%'=>{self.line("    cqo");self.line("    idiv rcx");self.line("    mov rax, rdx");},_=>{}}
            }
        } Ok(())
    }
    fn generate(stmts:&[Stmt])->Result<String,String>{
        let mut g=Gen{asm:String::new(),vars:Default::default(),slots:0,label:0};
        g.line(".intel_syntax noprefix");g.line(".text");g.line(".globl main");g.line(".type main, @function");g.line("main:");g.line("    push rbp");g.line("    mov rbp, rsp");
        // Reserve aligned stack space for all declared locals.
        for s in stmts { if let Stmt::Let(n,_)=s {if g.vars.contains_key(n){return Err(format!("duplicate variable '{n}'"));}g.slots+=8;g.vars.insert(n.clone(),g.slots*8);} }
        let bytes=((g.slots*8+15)/16)*16;if bytes>0{g.line(&format!("    sub rsp, {bytes}"));}
        for s in stmts {match s {
            Stmt::Let(n,e)=>{g.expr(e)?;let off=g.vars[n];g.line(&format!("    mov QWORD PTR [rbp-{off}], rax"));},
            Stmt::Assign(n,e)=>{let off=*g.vars.get(n).ok_or_else(||format!("unknown variable '{n}'"))?;g.expr(e)?;g.line(&format!("    mov QWORD PTR [rbp-{off}], rax"));},
            Stmt::Print(e)=>{g.expr(e)?;g.line("    mov rsi, rax");g.line("    lea rdi, [rip + .Lara_fmt]");g.line("    xor eax, eax");g.line("    call printf@PLT");},
            Stmt::Return(e)=>{g.expr(e)?;g.line("    leave");g.line("    ret");}
        }}
        g.line(".size main, .-main");g.line(".section .rodata");g.line(".Lara_fmt:");g.line("    .string \"%ld\\n\"");g.line(".section .note.GNU-stack,\"\",@progbits");
        Ok(g.asm)
    }
}

fn main() {
    if let Err(e)=run(){eprintln!("ara: error: {e}");std::process::exit(1);}
}
fn run()->Result<(),String>{
    let args:Vec<String>=env::args().skip(1).collect();
    if args.len()<2 || args[0]!="build" {return Err("usage: ara build <file.ara> [-o executable] [--emit-asm]".into());}
    let input=PathBuf::from(&args[1]);let mut output=PathBuf::from(input.file_stem().unwrap_or_default());let mut emit=false;let mut i=2;
    while i<args.len(){match args[i].as_str(){"-o"=>{i+=1;if i>=args.len(){return Err("missing path after -o".into());}output=PathBuf::from(&args[i]);},"--emit-asm"=>emit=true,"-O0"|" -O2"|"-O3"|"--target" => {},x=>return Err(format!("unknown option '{x}'"))}i+=1;}
    let source=fs::read_to_string(&input).map_err(|e|format!("cannot read {}: {e}",input.display()))?;
    let tokens=Lexer::new(&source).tokens()?;let ast=Parser{ts:tokens,p:0}.program()?;let asm=Gen::generate(&ast)?;
    let asm_path=PathBuf::from(format!("{}.s",output.display()));fs::write(&asm_path,&asm).map_err(|e|format!("cannot write assembly: {e}"))?;
    if emit {println!("wrote {}",asm_path.display());return Ok(());}
    let status=Command::new("cc").arg("-no-pie").arg(&asm_path).arg("-o").arg(&output).status().map_err(|_|"could not start 'cc'; install a system linker/compiler driver or use --emit-asm".to_string())?;
    if !status.success(){return Err("native linking failed".into());}
    println!("built {}",output.display());Ok(())
}
