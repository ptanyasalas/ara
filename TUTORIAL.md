# ARA tutorial (prototype)

This tutorial describes only syntax implemented by the current compiler. It is not a promise that planned language features already work.

## Requirements

- Rust stable and Cargo
- Linux x86-64 (Ubuntu on WSL2 is suitable)
- A system C compiler/linker driver named `cc` (for example, GCC)

## Build the compiler

```sh
cargo build --release
```

## First program

Create `hello.ara`:

```ara
fn main() -> i32 {
    print(42);
    return 0;
}
```

Compile and run:

```sh
cargo run --release -- build hello.ara -o hello
./hello
```

Expected output: `42`.

## Variables and arithmetic

```ara
fn main() -> i32 {
    i64 a = 100;
    i64 b = 25;
    i64 sum = a + b;
    i64 difference = a - b;
    i64 product = a * b;
    i64 quotient = a / b;
    i64 remainder = a % b;

    print(sum);
    print(difference);
    print(product);
    print(quotient);
    print(remainder);
    return 0;
}
```

Variables have explicit signed 64-bit integer type. A variable must be initialized before it is read or assigned.

## Reassigning a variable

```ara
fn main() -> i32 {
    i64 value = 10;
    value = value * 3 + 2;
    print(value);
    return 0;
}
```

## Parentheses and precedence

Multiplication, division, and remainder bind more tightly than addition and subtraction. Parentheses can make grouping explicit.

```ara
fn main() -> i32 {
    i64 result = (2 + 3) * 4;
    print(result);
    return 0;
}
```

## Inspect generated assembly

```sh
cargo run --release -- build hello.ara --emit-asm
```

This writes an assembly file instead of linking an executable.

## Current limitations

The prototype currently supports one `main` function, `i64` locals, integer arithmetic, `print`, and `return`. It does **not** yet implement conditionals, loops, user-defined functions, floating-point types, strings, arrays, pointers, vectors, matrices, tensors, modules, exceptions, or optimization levels above `-O0`. Do not assume these features compile.

Integer overflow is not trapped. Division by zero and signed division overflow can fault. The generated program uses the Linux x86-64 System V ABI and calls the C library's `printf`.

## Updating the repository

From your local clone:

```sh
git pull
cargo build --release
```
