# ARA

ARA is an experimental statically typed systems language focused on native performance and mathematical computing.

## Current status

**Milestone 1: integer control-flow compiler prototype.** The compiler is written in Rust. It includes a lexer, parser, a small AST, symbol handling, and a handwritten x86-64 assembly emitter. This is not yet the complete ARA specification.

Supported today:
- `fn main() -> i32`
- `i64` local variables and assignments
- integer expressions with `+`, `-`, `*`, `/`, `%` and parentheses
- `print(expression)` for signed 64-bit integers
- `return expression`
- `if` / `else` blocks
- `while` loops
- C-style `for` loops, including optional initializer, condition, and step
- `break` and `continue`
- Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`; logical `&&`, `||`, `!`; `true` and `false` literals (lowered to integer 1 and 0)
- Linux x86-64 assembly output and native linking through the system `cc` driver

Not implemented yet: floating-point types and operations, strings as values, arrays, pointers, vectors, matrices, tensors, modules, bounds checks, overflow traps, parallelism, ARM64, optimization passes, and a complete type checker.

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

## Control flow example

See [`examples/control_flow.ara`](examples/control_flow.ara) for `if`/`else`, `while`, and `for` syntax. Build it with:

```sh
cargo run --release -- build examples/control_flow.ara -o control_flow
./control_flow
```

## Important prototype limitations

- Only one function (`main`) is accepted; user-defined functions and parameters are not implemented.
- All locals are function-wide and must have unique names; lexical block scopes are not implemented.
- `&&` and `||` currently evaluate both operands (they do not short-circuit).
- Boolean expressions are represented as integer 0/1; there is no declared `bool` type yet.
- Loop control supports `break` and `continue`; these are compile-time checked to be inside a loop.
- Locals currently use signed 64-bit storage. The declared `i32` return type is lowered to the platform's 32-bit process exit status.
- Integer arithmetic uses x86-64 machine operations. Overflow is **not yet trapped** in this prototype.
- Division by zero and signed division overflow currently follow the processor's fault behavior.
- Local variables are stack allocated. There is no heap allocator or pointer syntax yet.
- The assembly emitter targets the System V x86-64 ABI and uses `printf` for integer output.
- Only `-O0` is currently accepted. `-O2` and `-O3` are rejected until real optimization passes exist.

ARA's long-term design includes a custom backend, static and dynamic tensor shapes, explicit memory management, SIMD, parallel execution, and GPU support. Those features must not be considered available until implemented and tested.
