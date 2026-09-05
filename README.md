# S+N++

**S+N++** एक readable, statically-oriented systems language का Rust-based compiler/runtime project है। इसका लक्ष्य सरल syntax, स्पष्ट diagnostics, safe ownership और practical concurrency है।

## Version 1.0 milestone

इस milestone में repository का executable अब thin binary entrypoint और testable library crate में विभाजित है। Current implementation में lexer, parser, AST, type checker, definite-assignment analysis, ownership/move analysis, bytecode compiler, stack VM और CLI workflow मौजूद हैं।

Language features में functions, typed parameters, `Int`, `String`, `Bool`, `Unit`, `Channel` और `Thread` values, arithmetic/comparison operators, `if`, `else`, `while`, `return`, lexical block scope, shadowing, branch-sensitive initialization, recursive calls, immutable/mutable reference syntax, ownership moves, `spawn`, `join`, `channel`, `send` और `receive` शामिल हैं।

## Worker-channel example

Workers को अब channel handles और अन्य values के साथ launch किया जा सकता है:

```snp
fn worker(ch: Channel) {
    send(ch, 99);
}

fn main() {
    let ch = channel();
    let task = spawn("worker", ch);
    join(task);
    print(receive(ch));
}
```

`spawn("worker", arguments...)` named function को अलग OS thread पर चलाता है और बाकी arguments forward करता है। `join(handle)` completion की प्रतीक्षा करता है। `channel()` synchronized message channel बनाता है।

## Build and test

Rust और Cargo install होने के बाद:

```bash
cargo test
cargo build --release
```

## Run

```bash
cargo run -- run examples/hello.snp
cargo run -- check examples/hello.snp
cargo run -- repl
```

## Architecture

```text
S+N++ source
    -> lexer and tokens
    -> recursive-descent parser
    -> AST
    -> type / definite-assignment / ownership checks
    -> bytecode compiler
    -> stack-based VM
    -> runtime services: threads and synchronized channels
```

The crate is currently organized as a testable library plus a thin binary entrypoint. The next major production milestones are explicit closure capture, richer diagnostics, standard library modules, serialized bytecode and an LLVM/native backend.

विस्तृत language rules और milestone details के लिए [SPEC.md](SPEC.md) देखें।
