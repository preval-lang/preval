<p align="center">
  <img src="https://raw.githubusercontent.com/preval-lang/assets/refs/heads/main/logo.png" alt="Preval logo" width="200"/>
</p>

<h1 align="center">Preval</h1>
<p align="center">A functional programming language with aggressive partial evaluation of side effects</p>

> [!WARNING]
> In its current state, Preval is a research language and as such performance is not a priority. The language is designed with potential performance in mind however the VM implementation doesn't prioritize performance or do any optimizations beyond partial evaluation.

## Installation
Requires [Rust](https://www.rust-lang.org/tools/install)
```bash
git clone https://github.com/preval-lang/preval
cd preval
cargo run
```

## What makes Preval different?
The flagship feature of Preval is its partial evaluator. 
It's typical to see partial evaluators in modern languages like Rust and C++ as a form of optimization. 
A partial evaluator is responsible for running code that doesn't depend on any runtime values at compile time to improve runtime performance.

For example, a partial evaluator would turn this function:
```rust
fn pi(): f32 {
  return 22/7;
}
```
Into this:
```rust
fn pi(): f32 {
  return 3.14
}
```
This theoretically improves the performance of the function because the division is moved from runtime to compile time. Preval's partial evaluator follows the same basic concept.

Preval's entry point looks like this:
```rust
fn main(compile_io: IO, io: IO) {

}
```
The `IO` type is empty and doesn't store any information. All effectful functions in preval accept an `IO` as a parameter. Take for example the print function:
```rust
fn print(io: IO, message: String) {}
```
The utility of the `IO` type comes from the partial evaluator. `compile_io` is treated as known at compile time, while `io` is treated as unknown.
```rust
fn main(compile_io: IO, io: IO) {
  print(io, "Hello, run time world!");
  print(compile_io, "Hello, compile time world!");
}
```
That means that in the example above, all the dependencies of the first `print` call are known at compile time, so the function can be executed! 
Compiling the program results in `Hello, compile time world!` being printed and running the program results in `Hello, run time world`. 
The partial evaluator eliminated the first call early by doing it at compile time.

None of the examples above crossed function call boundaries, because I wanted to keep the examples simple. However Preval's partial evaluator does, like any good partial evaluator, cross function boundaries and create specialised versions of functions for specific inputs.
## Why this is powerful
### Compile-time code checks
Allowing full access to the language and environment at compile time can allow library authors to improve the user experience of their libraries.
For example, SQL clients could validate queries at compile time. This is possible in other languages and is popular in Rust however the procedural macro system is tedious to use and delecate. 
In Preval, it's as straightforward as doing a check in the same way you would at runtime, then letting the partial evaluator handle moving it to compile time.
### Reflection
Preval plans to support reflection and generics. Reflection allows code to iterate over members of types like structs and enums. 
Reflection is not typically included in performance-focused programming languages because of its poor runtime performance. 
In Preval, this doesn't matter since the reflection can be automatically moved to compile time by the partial evaluator.
Reflection is by far the easiest metaprogramming technique to use and was only avoided in mainstream languages for performance reasons.
## Roadmap
### 0.1
- [x] Local variables
- [ ] Primitives u8-64 i8-64 String IO
- [x] If statements
- [ ] C interop
### 0.2
- [ ] Mutable state type
- [ ] First class functions
- [ ] `unsafe` marking
- [ ] Unsafe global io variable for calls to pure FFI functions
### 0.3
- [ ] Structs
- [ ] Tagged unions
- [ ] Traits
### 0.4
- [ ] Reflection
- [ ] Generics
### Beyond
- [ ] Native compilation