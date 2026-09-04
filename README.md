# S+N++

**S+N++** एक readable, statically-oriented systems language का शुरुआती compiler/runtime prototype है। इसका design लक्ष्य सरल syntax, स्पष्ट diagnostics और आगे चलकर safe memory तथा native performance है।

## Current Version 0.1

इस repository में Rust implementation निम्न सुविधाएँ देती है:

- Lexer और tokenization
- Function declarations और calls
- Integer, string और boolean values
- Variable declaration (`let x = ...`)
- Arithmetic तथा comparison operators
- `if`, `else`, `while` और `return`
- Built-in `print`
- AST-based execution
- REPL और `check` command

यह अभी **tree-walk execution prototype** है। Bytecode VM, ownership checker, LLVM AOT और JIT इसके बाद के milestones हैं।

## Build

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

Example:

```snp
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn main() -> Int {
    print(add(20, 22));
    return 0;
}
```

## Architecture roadmap

```text
Source -> Lexer -> Parser -> AST -> Semantic Checks -> HIR/MIR
       -> Bytecode VM (development) or LLVM AOT (production)
       -> S+N++ Runtime
```

विस्तृत Version 1 scope के लिए [SPEC.md](SPEC.md) देखें।
