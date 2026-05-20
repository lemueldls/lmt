# LMT

LMT (pronounced "limit"), the Language of Meaning & Types, is a proof-oriented meta-programming and specification language. It is designed so
that types and values live in the same logical space, letting programs express contracts, refinements, and proof
obligations that are checked at compile time.

The core idea is simple: source code is parsed into a typed intermediate form, checked bidirectionally, and then
translated into verification conditions for an SMT solver such as cvc5. That makes LMT a language for writing
specifications first, then proving that implementations satisfy them.

## What LMT is for

LMT is meant to be the specification layer that sits alongside ordinary codebases. It can describe invariants,
preconditions, postconditions, refinements, and synthesis goals without forcing a separate formal-methods workflow.

Its goals are:

- make program properties explicit and machine-checkable
- keep specifications close to the source they describe
- support compile-time verification instead of runtime assertions
- integrate with multiple host languages instead of replacing them
- support synthesis when a contract describes what must exist, but not how to build it

## Core model

LMT treats types as sets and values as singleton sets. Type checking becomes a question of set inclusion, and refinement
types are used to represent predicates over values. The universal top type is `Any`, while the bottom type is `Never` or
`!`.

```lmt
let Positive = Int | it > 0
let SmallEven = Int | it > 0 and it < 20 and it % 2 == 0

let x: Positive = 7
let y = 12 :: SmallEven
```

`x` is checked against a refinement type, and `y` uses verification ascription (`::`) to force an explicit proof
obligation.

The compilation pipeline is organized around three steps:

1. Parse source text into an abstract syntax tree.
2. Run bidirectional type checking to infer or verify expressions.
3. Emit verification conditions for an SMT solver.

## Host language integration

LMT is designed to work with host languages, not just as a standalone notation. Host code can carry embedded
specifications in comments or annotation blocks, and the compiler extracts those contracts, matches them to host
declarations, and checks that the implementation satisfies them.

```rust
/***lmt
  * let Byte = Int | it >= 0 and it <= 255
  * clamp_to_byte(x: i32) = Byte
 ***/
fn clamp_to_byte(x: i32) -> u8 {
	if x < 0 {
		0
	} else if x > 255 {
		255
	} else {
		x as u8
	}
}
```

This lets LMT describe real functions, modules, and APIs in the language they are already written in. The host compiler
remains responsible for ordinary compilation, while LMT adds a verification layer for the behavior that matters.

The intent is cross-language adoption, so the same verification model can be applied to different ecosystems without
forcing developers to rewrite everything in LMT.

```lmt
let NonZero = Int | it != 0

let safe_div(a: Int, b: NonZero): Int = a / b
```

This encodes the precondition (`b` must be non-zero) directly in the function signature.

## Cross-language program synthesis

LMT also treats some specifications as synthesis problems. A hole can mean two different things:

- `?` asks for a value that fits the local typing context.
- `??` asks the compiler to search for an implementation or witness that satisfies the constraint.

```lmt
let target = Int | it > 10 and it < 20
let a: target = ?

let square(n: Int) = n * n
let root: Int | square(it) == 16 = ??
```

In synthesis mode, the compiler can translate the goal into an enumerative search or a SyGuS-style query and delegate
the search to the solver backend. This is useful when the contract is stronger than a simple type annotation and the
missing piece should be discovered, not hand-written.

That makes LMT useful for cross-language program synthesis as well. The same constraint language can describe a result
needed by a host function, a derived value in a specification block, or a proof witness that must be found
automatically.

## Built-in directions

LMT maps its core datatypes to familiar SMT theories, including integers, reals, booleans, strings, arrays, sets,
sequences, and bit-vectors. The language is intended to stay close to solver semantics so that verification conditions
remain precise and predictable.

```lmt
let UnsignedByte = BitVec(8)
let Count = Int | it >= 0
let Names = List(String)
```

## Status

LMT is still a draft language. The current focus is on the parser, checker, solver integration, diagnostics, and editor
support needed to make the specification workflow practical.

## License

This project is licensed under [Apache License 2.0](https://choosealicense.com/licenses/apache-2.0/). See the [LICENSE](LICENSE) file for details.
