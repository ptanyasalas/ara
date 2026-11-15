# ARA

ARA is an experimental statically typed systems language focused on native performance and mathematical computing.

## Current status

**Milestone 0: integer compiler prototype.** The compiler is written in Rust. It includes a lexer, parser, a small AST, symbol handling, and a handwritten x86-64 assembly emitter. This is not yet the complete ARA specification.

Supported today:
- `fn main() -> i32`
- `i64` local variables and assignments
- integer expressions with `+`, `-`, `*`, `/`, `%` and parentheses
- `print(expression)` for signed 64-bit integers
- `return expression`
- Linux x86-64 assembly output and native linking through the system `cc` driver

Not implemented yet: floating point, strings as values, arrays, pointers, vectors, matrices, tensors, loops, conditionals, modules, bounds checks, overflow traps, parallelism, ARM64, optimization passes, and a complete type checker.

## Build the compiler

Requirements: Rust stable, Cargo, and (for native linking) a Linux x86-64 environment with a system `cc` driver and standard C library.

```sh
cargo build --release
```

## Write your first ARA program

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

Expected output:

```text
42
```

Inspect generated assembly:

```sh
cargo run --release -- build hello.ara --emit-asm
```

## Syntax example

```ara
fn main() -> i32 {
    i64 a = 20;
    i64 b = 22;
    i64 result = a + b;
    print(result);
    return 0;
}
```

## Important prototype limitations

- Only one function (`main`) is accepted.
- Locals currently use signed 64-bit storage. The declared `i32` return type is lowered to the platform's 32-bit process exit status.
- Integer arithmetic uses x86-64 machine operations. Overflow is **not yet trapped** in this prototype.
- Division by zero and signed division overflow currently follow the processor's fault behavior.
- Local variables are stack allocated. There is no heap allocator or pointer syntax yet.
- The assembly emitter targets the System V x86-64 ABI and uses `printf` for integer output.
- The `-O0`, `-O2`, and `-O3` flags are accepted as placeholders; optimization passes are not implemented yet.

ARA's long-term design includes a custom backend, static and dynamic tensor shapes, explicit memory management, SIMD, parallel execution, and GPU support. Those features must not be considered available until implemented and tested.
