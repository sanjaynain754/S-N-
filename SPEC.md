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

Current implementation source को tokens और AST में process करके bytecode instructions में compile करती है और stack-based VM पर चलाती है। अगली versions में HIR/MIR, ownership analysis और LLVM AOT backend जोड़े जाएँगे। हर चरण existing programs और diagnostics tests को preserve करेगा।

## Ownership subset

Version 1 का पहला ownership rule यह है कि `Int`, `Bool` और `Unit` copyable हैं, जबकि `String` और भविष्य के heap objects owned values हैं। किसी owned value को by-value function argument या दूसरे variable में देने पर उसका ownership move होता है। Moved variable को दोबारा उपयोग करने पर compiler `use of moved value` diagnostic देता है। `print` को read-only use माना गया है। Borrow syntax और `.clone()` API अगले milestone में formalize होंगे।
