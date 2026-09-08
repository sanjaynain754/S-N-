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

## Advanced scope safety

हर function call को अपना local environment और call frame मिलता है, इसलिए recursive calls में parameters और locals एक-दूसरे को overwrite नहीं करते। Ownership analysis branches को conservatively merge करता है: यदि किसी branch में owned variable move हो गया, तो join point पर variable को moved माना जाता है। इससे unsafe path छिप नहीं सकता। Closure capture और synchronized shared state अभी design phase में हैं; Version 1 में threads केवल named zero-argument functions को isolated environments में चलाते हैं।

## Synchronized channels

`channel()` एक synchronized message channel बनाता है। `send(ch, value)` value को channel में रखता है और `receive(ch)` अगली value आने तक सुरक्षित रूप से wait करता है। Channel handles copyable runtime capabilities हैं; payload ownership rules के अनुसार move हो सकती है। इस milestone में channel round-trip और runtime synchronization मौजूद है। Named zero-argument `spawn` API की सीमा के कारण worker को channel argument देना अभी अगला API refinement है।

S+N++ में arbitrary implicit closures अभी enabled नहीं हैं। जब closure syntax जोड़ी जाएगी, capture mode explicit होगा—`move` capture या immutable borrow capture—ताकि hidden shared mutable state न बने।

## Version 1.0 architecture milestone

The executable now uses a thin binary entrypoint and a library crate, allowing the compiler/runtime to be tested independently of the CLI. The runtime exposes typed `Channel` and `Thread` handles. `spawn("worker", argument...)` launches a named function on an OS thread and forwards the remaining values as function arguments; `join(handle)` waits for completion.

A worker can therefore communicate through a channel:

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

## Core Upgrade: module ownership and artifacts

The compiler implementation is now separated into dedicated AST, lexer, parser, typecheck, runtime and CLI modules. The runtime exposes a validated bytecode artifact builder. `build <source.snp>` emits `<source.snp>.snpbc` beginning with the `SNPBC1` format marker, while `check` runs parsing and semantic validation without execution. The current artifact is a compiler output/debug representation; direct artifact loading will be added before it is treated as a stable distribution format.

## Standard Library Foundation

S+N++ source files may declare standard-library imports before function declarations:

```snp
import std.io;
import std.concurrency;
```

Module paths are dotted identifiers and may end with an optional semicolon. The current resolver recognizes `std.io`, `std.string`, `std.collections`, and `std.concurrency`. The first three establish the initial library namespaces; `std.concurrency` documents the existing `channel`, `send`, `receive`, `spawn`, and `join` runtime surface. Unknown modules are rejected before type checking and execution. The registry is deliberately explicit so a future package manager can add versioned external modules without changing the language grammar.

## `std.string` and `std.collections` APIs

The first concrete standard-library APIs are available after importing the corresponding modules. String helpers are pure operations: `str_len(String) -> Int`, `str_contains(String, String) -> Bool`, `str_upper(String) -> String`, `str_lower(String) -> String`, `str_trim(String) -> String`, and `str_concat(String, String) -> String`.

Collections use an immutable-style list surface in this milestone. `list(value...) -> List` constructs a list, `list_push(List, value) -> List` returns a new list with one appended value, `list_len(List) -> Int` returns its length, and `list_get(List, Int)` returns the selected value or a runtime bounds error. List values are non-copyable for ownership analysis, while these standard-library operations borrow their input and preserve the original list.

## Advanced Collections

`std.collections` now supports immutable-style list updates and maps. `list_set(List, Int, value) -> List` returns a copy with the selected element replaced and rejects negative or out-of-bounds indices. `list_concat(List, List) -> List` returns a new list containing both inputs in order.

`map(String, value...) -> Map` constructs a map from alternating string keys and values. `map_set(Map, String, value) -> Map` returns a copy with a key inserted or replaced. `map_get(Map, String)` returns the associated value and reports `map key not found` when absent. `map_has(Map, String) -> Bool` checks key presence. Map keys are currently strings, and all map updates preserve the original map.

## Package manager foundation

A project manifest is stored as `snp.toml`. The supported foundation format has a `[package]` section with `name`, `version`, and optional `entry` (default `src/main.snp`), plus a `[dependencies]` section mapping package names to local paths. `snp init <directory> [name]` creates a starter project, and `snp package-check [snp.toml]` validates the manifest, resolves local dependency manifests, checks the entry source, and validates import namespaces.

Imports beginning with `std.` are checked against the built-in standard-library registry. Other import roots must match a declared dependency, so a dependency named `util` can provide `util.math`. This milestone does not yet download packages or solve semver ranges; registry access, lockfiles, checksums, and reproducible dependency graphs remain future work.
