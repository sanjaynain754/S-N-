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

## Mutable references

`&mut T` syntax अब parser और type system में मान्य है। यह reference type ownership move नहीं करता और future mutation operations के लिए exclusive access marker है। इस milestone में `&mut` की syntax, type compatibility और owner preservation लागू हैं; reference के माध्यम से field/index mutation तथा simultaneous-borrow conflict diagnostics अगले ownership milestone में पूर्ण किए जाएँगे।

## Concurrency milestone

S+N++ अब real OS threads के लिए दो built-in operations देता है। `spawn("function_name")` zero-argument function को background thread में चलाकर a thread handle लौटाता है। `join(handle)` उस thread के समाप्त होने की प्रतीक्षा करता है और उसका result लौटाता है। Thread functions अलग VM instance और अलग local environment में चलते हैं; सामान्य local variables अपने-आप shared नहीं होते। Shared mutable state और channels को सुरक्षित रूप से expose करना अगली concurrency milestone का विषय है।

### Scope test matrix

Version 1 में function और variable scoping के लिए निम्न cases cover किए गए हैं: function-local variable isolation, function parameter availability, undefined variable rejection, assignment केवल पहले declared variable पर, nested conditional/loop visibility, function return type checking, duplicate function names, unknown function calls, argument count/type validation, moved-variable visibility और borrowed-owner validity। अभी explicit lexical shadowing policy, branch-join definite assignment, loop-carried initialization, closure capture, recursive stack isolation और shared-state synchronization tests बाकी हैं।

## Lexical blocks and definite assignment

Braced `if` और `while` bodies अब child lexical environments में execute होते हैं। Inner `let` declaration outer variable को shadow कर सकती है, लेकिन block समाप्त होने पर shadowed value बाहर leak नहीं होती। Existing outer variables पर assignment block के बाद visible रहता है।

Typed declarations जैसे `let count: Int;` बिना initializer के allowed हैं, लेकिन उनका उपयोग तब तक नहीं किया जा सकता जब तक सभी reachable control-flow paths पर assignment न हो। An `if` assignment को definite तभी माना जाता है जब `then` और `else` दोनों branches assign करें। Loop body में हुआ assignment loop के बाद definite नहीं माना जाता, क्योंकि loop zero बार भी चल सकता है।
