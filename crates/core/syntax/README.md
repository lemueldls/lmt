# lmt-syntax - Parser and Abstract Syntax Tree

Lexical analysis, parsing, and AST generation for LMT source code.

## Architecture

The syntax crate converts source text into an AST. It uses a recursive descent parser with token windowing and
integrates with Picante for incremental parsing.

```
Source Code
  ↓
Lexer
  ↓
Parser
  ↓
AST
  ↓
Picante DB
```

## Key Features

- Complete LMT grammar for expressions, statements, patterns, and type annotations
- Incremental parsing via Picante to re-parse only changed regions
- Error recovery that collects multiple errors
- Token windowing to improve performance on large files

## Grammar

- Identifiers: [a-zA-Z\_][a-zA-Z0-9_]\*
- Keywords: let, if, else, match, and, or, not, use
- Built-in Compiler Directives: Prefixed with @ (e.g., @panic, @assume, @assert, @check, @eval).

Operators and Precedence (high → low)

| Operator     | Description                              | Associativity                    |
| ------------ | ---------------------------------------- | -------------------------------- | ------------- |
| .            | Field Access / Method Call / Variant Tag | Left-to-right                    |
| ^            | Mathematical Exponentiation              | Right-to-left                    |
| -, not       | Unary Negation, Logical NOT              | Right-to-left                    |
| \*, /, %     | Multiplicative                           | Left-to-right                    |
| +, -         | Additive                                 | Left-to-right                    |
| &            | Type Intersection (Logical AND for sets) | Left-to-right                    |
| <, <=, >, >= | Relational                               | Left-to-right                    |
| ==, !=       | Equality                                 | Non-associative                  |
|              |                                          | Refinement Mapping ("such that") | Left-to-right |
| and          | Logical Conjunction                      | Left-to-right                    |
| or           | Logical Disjunction                      | Left-to-right                    |
| :            | Domain Restriction / Parameter Binding   | Right-to-left                    |
| ::           | Verification Ascription / Proof Assert   | Non-associative                  |

### Formal EBNF

```ebnf
Program             ::= Statement*

Statement           ::= LetDecl
                      | UseDecl
                      | ExprStmt

LetDecl             ::= "let" Identifier ( ":" Expr )? ( "=" Expr )?
UseDecl             ::= "use" Identifier ( "." Identifier )*

ExprStmt            ::= Expr ";"

Expr                ::= VerificationExpr

VerificationExpr    ::= LogicalOrExpr ( "::" LogicalOrExpr )*

LogicalOrExpr       ::= LogicalAndExpr ( "or" LogicalAndExpr )*

LogicalAndExpr      ::= RefinementExpr ( "and" RefinementExpr )*

RefinementExpr      ::= EqualityExpr ( "|" ( Identifier "->" )? Expr )?

EqualityExpr        ::= RelationalExpr ( ( "==" | "!=" ) RelationalExpr )*

RelationalExpr      ::= AdditiveExpr ( ( "<" | "<=" | ">" | ">=" ) AdditiveExpr )*

AdditiveExpr        ::= MultiplicativeExpr ( ( "+" | "-" ) MultiplicativeExpr )*

MultiplicativeExpr  ::= IntersectionExpr ( ( "*" | "/" | "%" ) IntersectionExpr )*

IntersectionExpr    ::= PowerExpr ( "&" PowerExpr )*

PowerExpr           ::= UnaryExpr ( "^" UnaryExpr )*

UnaryExpr           ::= ( "-" | "not" )? PrimaryExpr

PrimaryExpr         ::= Literal
                      | Identifier
                      | Hole
                      | CallExpr
                      | MatchExpr
                      | IfExpr
                      | BlockExpr
                      | VariantExpr
                      | ParenExpr

ParenExpr           ::= "(" Expr ")"

Literal             ::= IntegerLiteral | RealLiteral | StringLiteral | BooleanLiteral

Hole                ::= "?" | "??"

CallExpr            ::= PrimaryExpr "(" ( ArgumentList )? ")"
ArgumentList        ::= Expr ( "," Expr )*

BlockExpr           ::= "{" Statement* Expr? "}"

IfExpr              ::= "if" Expr BlockExpr ( "else" ( BlockExpr | IfExpr ) )?

MatchExpr           ::= "match" Expr "{" MatchArm* "}"
MatchArm            ::= Pattern "=>" ( Expr | BlockExpr ) ","?

VariantExpr         ::= "." Identifier ( "(" ArgumentList ")" )?

Pattern             ::= Literal
                      | Identifier
                      | "." Identifier ( "(" PatternList ")" )?
                      | "_"
PatternList         ::= Pattern ( "," Pattern )*
```

## Notes

- AST nodes include spans for precise diagnostics
- Token windowing and Picante integration aim to make edits fast for large files
