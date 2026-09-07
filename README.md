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
cargo run -- build examples/hello.snp
cargo run -- repl
cargo run -- run examples/stdlib_import.snp
cargo run -- check examples/stdlib_import.snp
cargo run -- run examples/string_collections.snp
cargo run -- check examples/string_collections.snp
cargo run -- run examples/advanced_collections.snp
cargo run -- check examples/advanced_collections.snp
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

The crate is organized as a testable library plus a thin binary entrypoint. Source files may begin with imports such as `import std.io;` and `import std.concurrency;`. The resolver currently recognizes `std.io`, `std.string`, `std.collections`, and `std.concurrency`; this registry is the contract that the future package manager will extend. `std.string` provides `str_len`, `str_contains`, `str_upper`, `str_lower`, `str_trim`, and `str_concat`. `std.collections` provides immutable-style `list`, `list_push`, `list_len`, `list_get`, `list_set`, `list_concat`, `map`, `map_set`, `map_get`, and `map_has` operations. Lists and maps are returned as new values when updated, so the original collection remains available under the ownership model. The AST, lexer, parser, type checker, runtime and CLI now have dedicated module implementations. `snp build file.snp` validates and emits a `.snpbc` bytecode artifact with the `SNPBC1` format marker; `snp check` performs frontend validation, while `snp run` executes through the bytecode VM. Future production milestones include loading serialized artifacts directly, explicit closure capture, richer diagnostics, standard library modules and an LLVM/native backend.

विस्तृत language rules और milestone details के लिए [SPEC.md](SPEC.md) देखें।
