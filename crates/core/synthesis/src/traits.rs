use lmt_parser::Stmt;

use crate::{Primitive, SynType};

struct FunctionSignature {
    name: String,
    arguments: Vec<Argument>,
    return_type: SynType,
}

struct Trait<'src> {
    name: String,
    functions: Vec<TraitFunction<'src>>,
}

struct TraitFunction<'src> {
    signature: FunctionSignature,
    default_body: Option<Vec<Stmt<'src>>>,
}

struct Argument {
    name: String,
    r#type: SynType,
}

pub fn add() -> Trait<'static> {
    Trait {
        name: "Add".to_string(),
        functions: vec![TraitFunction {
            signature: FunctionSignature {
                name: "add".to_string(),
                arguments: vec![
                    Argument {
                        name: "self".to_string(),
                        r#type: SynType::Primitive(Primitive::Number),
                    },
                    Argument {
                        name: "other".to_string(),
                        r#type: SynType::Primitive(Primitive::Number),
                    },
                ],
                return_type: SynType::Primitive(Primitive::Number),
            },
            default_body: None,
        }],
    }
}
