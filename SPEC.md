# ARA language specification (working draft)

This document distinguishes the intended language from the features currently implemented by the prototype.

## Design goals

- Statically typed, explicit declarations.
- Native executables and a custom compiler backend.
- Explicit memory management; no mandatory garbage collector.
- First-class vectors, matrices, and tensors.
- Predictable low-level control with safe defaults where practical.

## Current implementation: milestone 0

The current compiler accepts exactly one function:

```ara
fn main() -> i32 {
    i64 a = 20;
    i64 b = 22;
    i64 result = a + b;
    print(result);
    return 0;
}
```

Supported statements: `i64 name = expression;`, `name = expression;`, `print(expression);`, and `return expression;`.

Supported expressions: signed integer literals, local variables, unary `+` and `-`, parentheses, and binary `+`, `-`, `*`, `/`, `%`. Precedence follows conventional arithmetic rules.

The compiler emits GNU/AT&T-toolchain-compatible Intel-syntax x86-64 assembly for the System V ABI and links it using the host `cc` driver. It is currently intended for Linux x86-64.

## Planned language rules (not yet implemented)

### Types

- Integers: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`.
- Floating point: `f16`, `f32`, `f64`.
- Mathematical types: `vec<T, N>`, `mat<T, R, C>`, `tensor<T, ...>`, `complex<T>`, `quat<T>`.
- Dimensions may be compile-time constants or explicitly dynamic.

### Memory

- Locals use stack storage where appropriate.
- Dynamic allocation and deallocation will be explicit.
- No mandatory tracing garbage collector.
- Unsafe pointer operations will be explicitly marked.
- The compiler should diagnose uninitialized reads and invalid operations where statically detectable.

### Safety and arithmetic

- Integer overflow checks are intended by default, with explicit opt-outs for performance-sensitive code.
- Array bounds checks are intended by default.
- Unsafe operations and unchecked indexing will require explicit syntax.
- The exact overflow semantics and optimization rules must be specified before those features ship.

### Parallelism and math

- Explicit parallel constructs are planned.
- Automatic parallelization may be applied only when dependence analysis proves it safe.
- Matrix multiplication and tensor operations will lower through a dedicated math optimization pipeline.
- SIMD and GPU support are future milestones, not current capabilities.

## Backend roadmap

1. x86-64 custom instruction selection and machine-code/assembly emission.
2. Register allocation, calling conventions, object emission, and optimization passes.
3. ARM64 backend.
4. SIMD lowering and math kernels.
5. GPU compilation and runtime integration.

The project may use system assemblers/linkers as tools without using C or C++ to implement the compiler itself.
