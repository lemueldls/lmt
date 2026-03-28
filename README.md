# lmt

Highly theoretical systems language focused on a innovative approach to
compile-time state management, extendable type synthesis, and memory safety.

## Language Design

### Inspiration

- Rust <https://www.rust-lang.org/>
  - Editions
  - Feature flags
- Pyret <https://pyret.org>
  - Syntactic decisions for educational use.
  - `nothing` types.
- Jakt <https://github.com/SerenityOS/jakt/>
  - Strong, weak, and raw pointers.
- TypeScript <https://www.typescriptlang.org/>
  - Literals as types.
- Zig <https://ziglang.org/>
  - Types as values (only in macros).
- Verus <https://github.com/verus-lang/verus>
  - Static signature validation.
- Wasm Interface Types
  <https://component-model.bytecodealliance.org/design/wit.html>

### Safety

- Soundness
  - Memory safety
  - Thread safety
- Semver compliant design patterns
  - Backwards compatibility
    - Editions
  - Forward compatibility
    - Editions
    - Feature flags
    - Deprecation
    - Sealed types
    - Exhaustive enums

### Goals

- Live reloading of a compiled language.

### Identifiers

Identifiers are case-sensitive, and can only contain ASCII letters, numbers,
underscores, and dashes. Identifiers must start with a letter or underscore, and
end with a letter, number, or underscore. Dashes are only allowed between two
letters. (/^[A-Za-z](-[A-Za-z]|[A-Za-z0-9_])\*$/)

<!-- Allowing kebab-case identifiers may be a controversial for a systems language,
but with today's linting, formatting, and auto-completion tools, I think it's a
reasonable tradeoff to make the language more readable. -->

```lmt
one_of_many_identifiers_in_a_row

ONE_OF_MANY_IDENTIFIERS_IN_A_ROW

one-of-many-identifiers-in-a-row

ONE-OF-MANY-IDENTIFIERS-IN-A-ROW
```

### Const-ness

Const-ness is a property of a value, and not a type. This means that a value can
be const or non-const, and a type can be const or non-const.

### Macros

#### Syntax

Macros are used with the `@` symbol, then follow the syntax of a function call.

```lmt
@my-macro-name(arg1, arg2, arg3)
```

Macros start with an `@` symbol, so they can be easily distinguished from
functions, unlike in languages like C, where macros look just like functions, or
in Rust, where the `!` is at the end of the macro name, which is harder to
distinguish from a function call at a glance.

They look and might behave similar to Python decorators or Java annotations,
except they are not limited to functions, and can be used on any kind of syntax.

##### Attribute Macros

Attribute macros are macros that are applied to a syntax item, and can modify
the item they are applied to.

```lmt
@my-macro(args)
struct MyStruct {
   field: u32,
}
```

To create an attribute macro, use the `macro` keyword, and provide a function
body that returns a syntax item.

```lmt
macro my-macro(args) -> Item {
   return Item::Nothing;
}
```

##### Expression Macros

Expression macros can be used as expressions, and can inlined into other
expressions.

```lmt
let my-var = @my-macro(args);
```

#### Resolved (Complete) Context

Instead of macros using an AST, which provides information as series of lexical
tokens, lmt provide a "resolved" context of the code, which contains reference
locations, mutability, constness, metadata, type synthesis, and other
information that is useful for compile-time validation and code generation.

This allows macros to be implemented in a way that is more similar to a
**compiler plugin**, and enables a more powerful macro system than what is
possible with just an AST.

With a "resolved" context, we can access metadata about any identifier,
referenced from anywhere in the code, and access types of expressions, even if
they are not fully resolved yet.

<!-- However, this also implies the code passed in must be a valid syntax tree
understood by the lmt compiler, which doesn't allow inlining an XML document or
a JSON object directly into the code. This is a tradeoff we are willing to make
for the sake of simplicity and consistency. -->

Macros using resolved contexts can change internal behavior and add or remove
fields, depending on mutability, constness, and other context-specific compile
time information.

Macros are defined with the `macro` keyword, and can be applied to any kind of
syntax. They behave like functions that return any type serializable for the
compiler to understand.

```lmt
use lmt.macros.Item;

macro const-inline(item: Item) -> Option<Item> {
   return if let Some(value) = item.const-value() {
      match Item {
         Item::Field(..) => {
            item.hooks.inline(fn() => value);

            None
         },
         _ => @panic("unsupported item type"),
      }
   } else {
      Some(item)
   }
}
```

#### Afterthoughts

##### Side Effects

Macros are allowed to have side effects, and can modify the code they are
applied to, but cannot modify code outside of the context they are applied to.

<!-- ##### Abuse of Limited Functionality -->

### Type Synthesis

#### Typestate Inferencing

Typestate inferencing is a feature that allows the compiler see how a value is
used, checked, and modified over time, and infer its typestate, which can be
used to determine if a value is in a valid state at a certain point in the code,
and prevent invalid state transitions.

### Values as Types

Similar to TypeScript, when assigning a type to a value as a constant
(`as const`), the compiler will set the value of the type to the value of the
constant, instead of the type of the constant.

```ts
// TypeScript

const foo = 123;
//    ^ number
const bar = 123 as const;
//    ^ 123
```

In lmt, this the default behavior. With this, we can create partial and total
type checks, and use them to create compile-time validation for the typestate of
a value.

Whenever a function is called, it will always try to run the function at
compile-time until it reaches a non-constant value. This is extremely powerful
for compile-time error checking and code optimizations of constant values and
type synthesis.

```lmt
struct List<T> {
   ptr: raw T,
   @const-inline length: usize,
   @const-inline capacity: Option<usize>,

   fn get(&self, index: usize) -> &T {
      // Will be checked at compile-time if both `self.length` and `index`
      // are constants. If not, it will be checked and panic at runtime.
      if index >= self.length {
         @panic("index out of bounds");
      }

      let value = unsafe { self.ptr.add(index).get() };

      return value;
   }

   fn push(&mut self, value: T) {
      @cfg(debug: true)
      if let Some(capacity) = self.capacity {
         if self.length >= capacity {
            @warn("list capacity exceeded");
         }
      }

      self.ptr[self.length] = value;
      // still considered a constant value after use
      // at compile-time, by modifying the typestate.
      self.length += 1;
   }
}
```

### Tuples, Lists, and Arrays

In lmt, tuples, lists, and arrays are all the same thing, and all are written as
a comma separated list of values, surrounded by square brackets.

```lmt
let things = [1, 2, 3];
```

However, because behavior can be predetermined depending on mutability and
constant-ness, they can be represented differently depending on how they are
used.

Tuples can be represented as an immutable, fixed-size series of values. This
allows safe deconstruction of tuples, and an empty tuple to represent a void
value. Tuples can also hold values of different types.

```lmt
let numbers = [1, 2, 3];
let [a, b, c] = numbers; // safe: can never be out of bounds.

let literal = [1, "two", true];
random-use-of(literal); // uses the tuple at random.
let [int, str, bool] = literal; // safe: can never be out of bounds.

let nothing = [];
@assert(@size-of(nothing) == 0);
```

Lists can be represented as a mutable, variable-size series of values.

```lmt
let mut list = [1, 2, 3];
list.add(4);
let [a, b, c, d] = list; // safe: can never be out of bounds.
some-random-mutation(list); // modifies the list at random.
let [something] = list; // error: possible out of bounds access.
```

Arrays can be represented as a mutable, fixed-size series of values. This allows
for compile-time bounds checking, while still allowing for mutation. Arrays can
also hold values of different types.

```lmt

let mut array = &[1, "what", false];
some-random-mutation(&mut array); // modifies the array at random.
array[0] = 4; // error: possible out of bounds access.

let mut another-array = &[1, 2, 3]; // fixed-size and same types.
another-array.sorted().reversed(); // unrolls and combines functions into one.
```

Iterating over a list or array will always try to unroll the loop at
compile-time, and if the size of the list or array is known at compile-time, it
will be unrolled in-place, and not create a new list or array.

### Copy on Write

Copy on write is a feature that allows a value to be copied only when it is
modified. This allows for efficient memory usage, and allows for values to be
passed by reference, without the need for a garbage collector.

```lmt
let mut value = 123;
let mut reference = &value;
reference = 456; // copies on write

let mut value = 123;
let mut copy = value;
copy = 456; // copies on write
```

### Closure Capture

Closures can capture values by reference, and can be used to create a
"function-like" syntax for creating objects.

```lmt
let mut closure = fn() => value;
```

### Contextual Functions

When using a function in which the behavior is determined by the context it is
used in, such as mutability, lmt creates a "family" of functions that internally
call the same functions, but with different behavior depending on the context.

## Conventions

### Named Arguments

It's optional to use named arguments, and only recommended for macros and enums.

```lmt
function-name(value1, value2, value3)

@macro-name(arg1: value1, arg2: value2, arg3: value3)
```
