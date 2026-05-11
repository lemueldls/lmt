#[rust_sitter::grammar("lmt")]
pub mod grammar {
    use std::fmt;

    #[rust_sitter::language]
    #[derive(Debug, Clone, PartialEq)]
    pub struct Program {
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ";")]
            ()
        )]
        stmts: Vec<Item>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Item {
        // LetBinding {
        //     name: String,
        //     value: Expression,
        // },
        // FunctionDef {
        //     name: String,
        //     params: Vec<Parameter>,
        //     return_type: Type,
        //     body: Expression,
        // },
        // TypeAlias {
        //     name: String,
        //     ty: Type,
        // },
        // Assertion {
        //     predicate: Expression,
        // },
        Expr(Expression),
    }

    // #[derive(Debug, Clone, PartialEq)]
    // pub struct Parameter {
    //     pub name: String,
    //     pub ty: Option<Type>,
    // }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Type {
        Base(BaseType),
        Refined {
            #[rust_sitter::leaf(text = "{")]
            lbrace: (),
            base: BaseType,
            #[rust_sitter::leaf(text = "|")]
            pipe: (),
            // v: Identifier,
            predicate: Expression,
            #[rust_sitter::leaf(text = "}")]
            rbrace: (),
        },
        Named(TypedIdent),
        Arrow {
            #[rust_sitter::leaf(text = "(")]
            lparen: (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            params: Vec<Type>,
            #[rust_sitter::leaf(text = ")")]
            rparen: (),
            #[rust_sitter::leaf(text = "->")]
            arrow: (),
            return_type: Box<Type>,
        },
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum BaseType {
        #[rust_sitter::leaf(text = "Int")]
        Int,
        #[rust_sitter::leaf(text = "Bool")]
        Bool,
        #[rust_sitter::leaf(text = "Real")]
        Real,
        #[rust_sitter::leaf(text = "Type")]
        Type,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Expression {
        // Literals
        Int(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse::<i64>().unwrap())] i64),
        Real(
            #[rust_sitter::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse::<f64>().unwrap())]
            f64,
        ),
        #[rust_sitter::leaf(text = "true")]
        True,
        #[rust_sitter::leaf(text = "false")]
        False,

        // Identifiers
        Ident(TypedIdent),

        // Let binding
        Let {
            #[rust_sitter::leaf(text = "let")]
            let_kw: (),
            name: TypedIdent,
            #[rust_sitter::leaf(text = "=")]
            eq: (),
            value: Box<Expression>,
            body: Box<Expression>,
        },

        // Binary operations
        #[rust_sitter::prec_left(3)]
        Add(
            Box<Expression>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(3)]
        Sub(
            Box<Expression>,
            #[rust_sitter::leaf(text = "-")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(4)]
        Mul(
            Box<Expression>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(4)]
        Div(
            Box<Expression>,
            #[rust_sitter::leaf(text = "/")] (),
            Box<Expression>,
        ),

        // #[rust_sitter::prec_left(2)]
        // Cons(Box<Expression>, Box<Expression>),
        #[rust_sitter::prec_left(1)]
        Lt(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Le(
            Box<Expression>,
            #[rust_sitter::leaf(text = "<=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Gt(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Ge(
            Box<Expression>,
            #[rust_sitter::leaf(text = ">=")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Eq(
            Box<Expression>,
            #[rust_sitter::leaf(text = "==")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(1)]
        Ne(
            Box<Expression>,
            #[rust_sitter::leaf(text = "!=")] (),
            Box<Expression>,
        ),

        #[rust_sitter::prec_left(0)]
        And(
            Box<Expression>,
            #[rust_sitter::leaf(text = "&&")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(0)]
        Or(
            Box<Expression>,
            #[rust_sitter::leaf(text = "||")] (),
            Box<Expression>,
        ),
        // #[rust_sitter::prec_left(0)]
        // Implies(
        //     Box<Expression>,
        //     #[rust_sitter::leaf(text = "=>")] (),
        //     Box<Expression>,
        // ),

        // // Unary operations
        // #[rust_sitter::prec_right(6)]
        // Not(#[rust_sitter::leaf(text = "!")] (), Box<Expression>),
        // #[rust_sitter::prec_right(6)]
        // Neg(#[rust_sitter::leaf(text = "-")] (), Box<Expression>),

        // Function application
        #[rust_sitter::prec_left(10)]
        App {
            func: Box<Expression>,
            #[rust_sitter::leaf(text = "(")]
            lparen: (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            args: Vec<Expression>,
            #[rust_sitter::leaf(text = ")")]
            rparen: (),
        },

        // // Lambda
        // Lambda {
        //     #[rust_sitter::leaf(text = "(")]
        //     lparen: (),
        //     #[rust_sitter::delimited(
        //         #[rust_sitter::leaf(text = ",")]
        //         ()
        //     )]
        //     params: Vec<TypedIdent>,
        //     #[rust_sitter::leaf(text = ")")]
        //     rparen: (),
        //     #[rust_sitter::leaf(text = "=>")]
        //     arrow: (),
        //     body: Box<Expression>,
        // },

        // Match
        Match {
            #[rust_sitter::leaf(text = "match")]
            match_kw: (),
            expr: Box<Expression>,
            #[rust_sitter::leaf(text = "with")]
            with_kw: (),
            #[rust_sitter::repeat(non_empty = true)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = "|")]
                ()
            )]
            arms: Vec<MatchArm>,
        },

        // // Tuples
        // Tuple(
        //     #[rust_sitter::leaf(text = "(")] (),
        //     #[rust_sitter::delimited(
        //         #[rust_sitter::leaf(text = ",")]
        //         ()
        //     )]
        //     Vec<Expression>,
        //     #[rust_sitter::leaf(text = ")")] (),
        // ),

        // Lists
        List(
            #[rust_sitter::leaf(text = "[")] (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            Vec<Expression>,
            #[rust_sitter::leaf(text = "]")] (),
        ),
        // // Parenthesized
        // Paren(
        //     #[rust_sitter::leaf(text = "(")] (),
        //     Box<Expression>,
        //     #[rust_sitter::leaf(text = ")")] (),
        // ),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct TypedIdent {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        name: String,
        type_ann: Option<TypeAnnotation>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct TypeAnnotation {
        #[rust_sitter::leaf(text = ":")]
        colon: (),
        ty: Type,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Pattern {
        #[rust_sitter::leaf(text = "_")]
        Wild,
        Var(TypedIdent),
        Literal(Literal),
        Tuple(
            #[rust_sitter::leaf(text = "(")] (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            Vec<Pattern>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
        List(
            #[rust_sitter::leaf(text = "[")] (),
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = ",")]
                ()
            )]
            Vec<Pattern>,
            #[rust_sitter::leaf(text = "]")] (),
        ),
        // Cons(
        //     Box<Pattern>,
        //     #[rust_sitter::leaf(text = "::")] (),
        //     Box<Pattern>,
        // ),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Literal {
        Int(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse::<i64>().unwrap())] i64),
        Real(
            #[rust_sitter::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse::<f64>().unwrap())]
            f64,
        ),
        Bool(
            #[rust_sitter::leaf(pattern = r"true|false", transform = |v| v.parse::<bool>().unwrap())]
             bool,
        ),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct MatchArm {
        pattern: Pattern,
        #[rust_sitter::leaf(text = "=>")]
        arrow: (),
        body: Expression,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }

    #[rust_sitter::skip]
    struct Comment {
        #[rust_sitter::leaf(pattern = r"//[^\n]*")]
        _comment: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar::*;

    #[test]
    fn test_parse_literal_int() {
        let result = grammar::parse("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_literal_bool() {
        let result = grammar::parse("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_identifier() {
        let result = grammar::parse("foo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_addition() {
        let result = grammar::parse("1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_binding() {
        let result = grammar::parse("let x = 42; x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda() {
        let result = grammar::parse("x => x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple() {
        let result = grammar::parse("(1, 2, 3)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_list() {
        let result = grammar::parse("[1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_match() {
        let result = grammar::parse("match x with | 0 => 1 | y => y + 2");
        assert!(result.is_ok());
    }
}
