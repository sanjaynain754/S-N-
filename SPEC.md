# S+N++ Version 1 Specification

S+N++ का Version 1 एक छोटा, readable, statically-oriented language foundation है। Source files की extension `.snp` होगी। इस milestone का उद्देश्य reliable lexer, parser, AST evaluator, diagnostics और CLI है; native LLVM backend, ownership checker और JIT अगले milestones में आएँगे।

## Syntax

```snp
fn add(a: Int, b: Int) -> Int:
    return a + b

fn main() -> Int:
    x := 20
    y := 22
    print(add(x, y))
    return 0
```

Version 1 में function declarations, `let`/`:=` variable declarations, reassignment, integer and string literals, arithmetic/comparison operators, `if`, `while`, `return`, function calls और `print` built-in supported हैं। Blocks indentation के स्थान पर braces का उपयोग करते हैं ताकि पहला compiler छोटा और deterministic रहे; Python-like indentation syntax को future grammar revision में जोड़ा जाएगा।

## Commands

```text
snp run file.snp
snp check file.snp
snp repl
```

## Compilation roadmap

Current implementation source को tokens, AST और evaluator में process करती है। अगली versions में AST से HIR/MIR, bytecode VM, ownership analysis और LLVM AOT backend जोड़े जाएँगे। हर चरण existing programs और diagnostics tests को preserve करेगा।
