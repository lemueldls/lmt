use std::{
    fmt, fs,
    hash::{Hash, Hasher},
    iter,
    ops::{Deref, Range},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use ahash::AHasher;
use hashbrown::{HashMap, hash_map};
// use decimal::Decimal;
// use integer::Integer;
use lmt_number::{
    LmtInteger,
    fraction::{BigUint, Zero},
};
use lmt_parser::{BinaryOp, Block, Expr, Function, Literal, SimpleSpan, Span, Spanned, Stmt};

use super::SpanWithModuleId;
use crate::{
    Implication, ModuleId, ModuleMap, Primitive, Proof, SynType, SynTypeKind, TypeId,
    context::Context,
    scope::{Scope, ScopeId},
    temp,
};

pub struct Module {
    exports: Vec<SynType>,
}

pub struct FileHash {
    pub path: PathBuf,
    hash: u64,
}

pub enum FileHashCheckResult {
    Relevant,
    Outdated(String),
}

impl FileHash {
    pub fn check(&self) -> FileHashCheckResult {
        let mut state = AHasher::default();

        let source = fs::read_to_string(&self.path).unwrap();
        source.hash(&mut state);

        let hash = state.finish();

        if self.hash == hash {
            FileHashCheckResult::Relevant
        } else {
            FileHashCheckResult::Outdated(source)
        }
    }
}

pub struct SerializedModule {
    exports: Vec<SynType>,
    module_file_map: ModuleMap<FileHash>,
}

impl SerializedModule {
    // pub fn get_stale_module_ids(&mut self) -> ModuleMap<String> {
    //     let mut stale_modules = ModuleMap::<String>::with_capacity_from(&self.module_file_map);

    //     // self.module_file_map
    //     //     .iter()
    //     //     .enumerate()
    //     //     .filter_map(|(i, file_hash)| {
    //     //         match file_hash.check() {
    //     //             FileHashCheckResult::Relevant => None,
    //     //             FileHashCheckResult::Outdated(source) => Some(ModuleId(i as u32)),
    //     //         }
    //     //     })
    //     //     .collect()

    //     // for (i, file_hash) in self.module_file_map.iter().enumerate() {
    //     //     match file_hash.check() {
    //     //         FileHashCheckResult::Relevant => {}
    //     //         FileHashCheckResult::Outdated(source) => stale_modules.insert(i, source),
    //     //     }
    //     // }

    //     // dbg!(&stale_modules);

    //     stale_modules
    // }
}

#[repr(u8)]
pub enum HintFlag {
    None = 0,
    Constant = 1 << 0,
    Pure = 1 << 1,
}

pub struct Hint {
    flags: u8,
}

impl Default for Hint {
    fn default() -> Self {
        Self::new()
    }
}

impl Hint {
    pub fn new() -> Self {
        Self {
            flags: HintFlag::None as u8,
        }
    }

    pub fn set(&mut self, flag: HintFlag) {
        self.flags |= flag as u8;
    }

    pub fn is(&self, flag: HintFlag) -> bool {
        self.flags & flag as u8 != 0
    }
}

#[derive(Debug)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct ModuleSynthesis {
    module_id: ModuleId,
    // ident_types: Arc<RwLock<HashMap<SpanWithFileId, TypeId>>>,
    /// Maps the span of the identifier of a `let` declaration with all referring identifier's span
    pub ident_references: HashMap<SimpleSpan, Vec<SimpleSpan>>,
    /// Maps the span of an identifier with the original `let` declaration' identifier's span.
    pub ident_definitions: HashMap<SimpleSpan, SimpleSpan>,
    /// Maps the span of an identifier with the it's corresponding type.
    // pub ident_types: HashMap<Span, TypeId>,
    pub errors: Vec<Spanned<Box<str>>>,

    ident_stacks: HashMap<String, Vec<Spanned<String>>>,

    active_scopes: Vec<ScopeId>,
}

#[derive(Debug)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct FunctionContext {
    pub return_type: TypeId,
}

impl ModuleSynthesis {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            ident_references: HashMap::new(),
            ident_definitions: HashMap::new(),
            errors: Vec::new(),

            ident_stacks: HashMap::new(),
            active_scopes: Vec::new(),
        }
    }

    pub fn id(&self) -> ModuleId {
        self.module_id
    }

    pub fn eval_block(
        &mut self,
        spanned_block: Spanned<Block>,
        is_top_level: bool,
        context: &Context,
    ) -> TypeId {
        // let scope_id = context.scope_map.insert(Scope );

        let (block, span) = spanned_block.into_deref_spanned();

        let to_be_evaluated = self.hoist_block(block, context);
        let mut return_value = None;

        for stmt in to_be_evaluated {
            match stmt {
                Stmt::Error | Stmt::Import(..) | Stmt::Function(..) => unreachable!(),

                Stmt::Let(typed_ident, value) => {
                    // if is_top_level {
                    //     todo!("top-level let declarations")
                    // } else {
                    self.declare_let(typed_ident.into_deref(), value, context);
                    // }
                }
                Stmt::Expr(expr) => {
                    // if is_top_level {
                    //     todo!("top-level expressions")
                    // } else {
                    self.eval_expression(expr, context);
                    // }
                }
                Stmt::Return(expr) => return_value = Some(self.eval_expression(expr, context)),
            }
        }

        return_value.unwrap_or_else(|| nothing_at(span, context))
    }

    fn hoist_block(&mut self, block: Block, context: &Context) -> Vec<Stmt> {
        match block {
            Block::Multiline {
                return_typed_ident,
                stmts,
            } => {
                let mut to_be_evaluated = Vec::with_capacity(stmts.len());

                for spanned_stmt in stmts {
                    let (stmt, span) = spanned_stmt.into_deref_spanned();

                    match stmt {
                        Stmt::Error => {
                            self.throw_unknown_at(span, context);
                        }
                        Stmt::Import(_) => {
                            // todo!()
                        }
                        Stmt::Function(function) => self.declare_function(function, context),
                        Stmt::Let(..) | Stmt::Expr(..) | Stmt::Return(..) => {
                            to_be_evaluated.push(stmt)
                        }
                    }
                }

                to_be_evaluated
            }
            Block::Singleline { return_expr } => vec![Stmt::Expr(*return_expr)],
        }
    }

    // fn eval_function(&mut self, function: Function) -> FunctionContext {
    //     let mut return_type = None;
    //     for stmt in function.body {
    //         return_type = self.eval_statement(stmt);
    //     }

    //     FunctionContext {
    //         return_type: return_type.unwrap_or_else(|| nothing_at(function.signature.name.span())),
    //     }
    // }

    // pub fn eval_statement(&mut self, stmt: Spanned<Stmt>, context: &Context) -> Option<TypeId> {
    //     let stmt = stmt.into_deref();

    //     match stmt {
    //         Stmt::Error => todo!(),
    //         Stmt::Expr(expr) => {
    //             self.eval_expression(expr, context);

    //             None
    //         }
    //         Stmt::Let(typed_ident, value) => {
    //             self.declare_let(typed_ident, value, context);

    //             None
    //         }
    //         Stmt::Function(function) => {
    //             self.declare_function(function, context);

    //             None
    //         }
    //         Stmt::Import(path) => {
    //             dbg!(path);

    //             todo!()
    //         }
    //         Stmt::Return(expr) => Some(self.eval_expression(expr, context)),
    //     }
    // }

    fn declare_let(
        &mut self,
        typed_ident: lmt_parser::TypedIdent,
        value: Box<Spanned<Expr>>,
        context: &Context,
    ) {
        let name_spanned = typed_ident.ident;

        let (name, span) = name_spanned.deref_spanned();

        if let Some(idents) = self.ident_stacks.get_mut(name) {
            idents.push(name_spanned.map(|name| name.to_string()));
        } else {
            self.ident_stacks.insert(name.to_string(), vec![
                name_spanned.map(|name| name.to_string()),
            ]);
        }

        self.ident_references.insert(span, Vec::new());

        let syn_type = self.eval_expression(*value, context);
        self.define_type(&syn_type, span, context);
    }

    fn declare_function(&mut self, function: Function, context: &Context) {
        let name_spanned = function.signature.name.clone();
        let (name, span) = name_spanned.deref_spanned();

        if let Some(idents) = self.ident_stacks.get_mut(name) {
            idents.push(name_spanned.map(|name| name.to_string()));
        } else {
            self.ident_stacks.insert(name.to_string(), vec![
                name_spanned.map(|name| name.to_string()),
            ]);
        }

        self.ident_references.insert(span, Vec::new());

        context.register_and_define_type_at_span(
            SynType::new(Proof::EqualTo(Spanned::new(
                SynTypeKind::Function(function),
                span,
            ))),
            self.span_with_id(span),
        );
    }

    fn eval_expression(&mut self, expr: Spanned<Expr>, context: &Context) -> TypeId {
        let (expr, span) = expr.into_deref_spanned();

        let r#type = match expr {
            Expr::Error => self.throw_unknown_at(span, context),
            Expr::Literal(literal) => {
                context.register_type(SynType::new(Proof::EqualTo(Spanned::new(
                    SynTypeKind::Constant(literal),
                    span,
                ))))
            }
            Expr::Ident(ident) => {
                let (name, span) = ident.deref_spanned();

                if let Some(ident_stack) = self.ident_stacks.get_mut(name) {
                    let origin_span = ident_stack.last().unwrap().span();

                    self.ident_references
                        .get_mut(&origin_span)
                        .unwrap()
                        .push(span);
                    self.ident_definitions.insert(span, origin_span);

                    // context.get_type_id_from_span(self.span_with_id(span))
                    context.get_type_id_from_span(self.span_with_id(origin_span))
                } else {
                    return self.throw_unknown_at(span, context);
                }
            }
            Expr::Block(block) => self.eval_block(Spanned::new(block, span), false, context),
            Expr::Match(match_) => {
                let expr = self.eval_expression(match_.expr, context);

                todo!()
            }
            Expr::List(_) => todo!(),
            Expr::Binary(left, binary_op, right) => {
                let left_span = left.span();
                let right_span = right.span();

                match binary_op {
                    BinaryOp::Add => self.binary_op(left, right, span, temp::add, context),
                    BinaryOp::Sub => todo!(),
                    BinaryOp::Mul => todo!(),
                    BinaryOp::Div => self.binary_op(left, right, span, temp::divide, context),
                    BinaryOp::Equal => {
                        let left_type_id = self.eval_expression(*left, context);
                        let left_type = context.get_type_from_id(left_type_id);

                        let right_type_id = self.eval_expression(*right, context);
                        let right_type = context.get_type_from_id(right_type_id);

                        let left_proof = left_type.proof.apply(&Proof::EqualTo(
                            right_type.proof.equal_to().unwrap().clone(),
                        ));
                        let left_proof = match left_proof {
                            Ok(left_proof) => left_proof,
                            Err(err) => {
                                return self.throw_unknown_at(span, context);
                            }
                        };

                        let right_proof = right_type.proof.apply(&Proof::EqualTo(
                            right_type.proof.equal_to().unwrap().clone(),
                        ));
                        let right_proof = match right_proof {
                            Ok(right_proof) => right_proof,
                            Err(err) => {
                                return self.throw_unknown_at(span, context);
                            }
                        };

                        let kind = SynTypeKind::Primitive(Primitive::Boolean);

                        context.register_type(
                            SynType::new(Proof::EqualTo(Spanned::new(kind, span)))
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(true),
                                    then_implies: left_proof.clone(),
                                    for_type: Spanned::new(left_type_id, left_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(true),
                                    then_implies: right_proof.clone(),
                                    for_type: Spanned::new(right_type_id, right_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(false),
                                    then_implies: left_proof.negate(),
                                    for_type: Spanned::new(left_type_id, left_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(false),
                                    then_implies: right_proof.negate(),
                                    for_type: Spanned::new(right_type_id, right_span),
                                }),
                        )
                    }
                    BinaryOp::NotEqual => todo!(),
                    BinaryOp::LessThan => {
                        let left_type_id = self.eval_expression(*left, context);
                        let left_type = context.get_type_from_id(left_type_id);

                        let right_type_id = self.eval_expression(*right, context);
                        let right_type = context.get_type_from_id(right_type_id);

                        // dbg!(&left_type, &right_type);
                        // dbg!(&left_proof, &right_proof);

                        let left_proof = left_type.proof.apply(&Proof::LessThan(
                            right_type.proof.equal_to().unwrap().clone(),
                        ));
                        let left_proof = match left_proof {
                            Ok(left_proof) => left_proof,
                            Err(err) => {
                                dbg!(err);
                                return self.throw_unknown_at(span, context);
                            }
                        };
                        let right_proof = right_type.proof.apply(&Proof::Or(
                            Box::new(Proof::GreaterThan(
                                dbg!(&left_type.proof).equal_to().unwrap().clone(),
                            )),
                            Box::new(Proof::EqualTo(left_type.proof.equal_to().unwrap().clone())),
                        ));
                        let right_proof = match right_proof {
                            Ok(right_proof) => right_proof,
                            Err(err) => {
                                dbg!(err);
                                return self.throw_unknown_at(span, context);
                            }
                        };

                        let kind: Option<SynTypeKind> = try {
                            let left_literal = left_proof.literal()?;
                            let right_literal = right_proof.literal()?;

                            let less_than = temp::less_than(left_literal, right_literal).ok()?;

                            SynTypeKind::Constant(Literal::Boolean(less_than))
                        };
                        let syn_type = SynType::new(Proof::EqualTo(Spanned::new(
                            kind.unwrap_or(SynTypeKind::Primitive(Primitive::Boolean)),
                            span,
                        )));

                        context.register_type(
                            syn_type
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(true),
                                    then_implies: left_proof.clone(),
                                    for_type: Spanned::new(left_type_id, left_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(true),
                                    then_implies: right_proof.clone(),
                                    for_type: Spanned::new(right_type_id, right_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(false),
                                    then_implies: left_proof.negate(),
                                    for_type: Spanned::new(left_type_id, left_span),
                                })
                                .with_implication(Implication {
                                    if_equal: Literal::Boolean(false),
                                    then_implies: right_proof.negate(),
                                    for_type: Spanned::new(right_type_id, right_span),
                                }),
                        )
                    }
                    BinaryOp::LessThanOrEqual => todo!(),
                    BinaryOp::GreaterThan => todo!(),
                    BinaryOp::GreaterThanOrEqual => todo!(),
                }
            }
            Expr::Call(ident_spanned, args_spanned) => todo!(),
            // Expr::If(condition, then, otherwise) => {
            //     let condition_type = self.eval_expression(*condition);

            //     let mut true_implications = Vec::new();

            //     for implication in condition_type.borrow().implications.iter() {
            //         if let Literal::Boolean(true) = implication.if_equal {
            //             true_implications.push((
            //                 implication.for_type.clone(),
            //                 implication
            //                     .for_type
            //                     .borrow()
            //                     .proof
            //                     .apply(&implication.then_implies.negate())
            //                     .unwrap(),
            //             ));

            //             implication
            //                 .for_type
            //                 .borrow_mut()
            //                 .proof
            //                 .apply(&implication.then_implies)
            //                 .unwrap();
            //         }
            //     }

            //     let then_type = self.eval_expression(*then);

            //     for (for_type, proof) in true_implications {
            //         for_type.borrow_mut().proof = proof;
            //     }

            //     let otherwise_type = self.eval_expression(*otherwise);

            //     todo!()
            // }
        };

        self.define_type(&r#type, span, context);

        r#type
    }

    fn define_type(&mut self, r#type: &TypeId, span: SimpleSpan, context: &Context) {
        context.define_type_id_at_span(*r#type, self.span_with_id(span))
    }

    fn binary_op(
        &mut self,
        left: Box<Spanned<Expr>>,
        right: Box<Spanned<Expr>>,
        span: SimpleSpan,
        operation: fn(left: &Literal, right: &Literal) -> Result<Literal, Box<str>>,
        context: &Context,
    ) -> TypeId {
        let left_type_id = self.eval_expression(*left, context);
        let left_type = context.get_type_from_id(left_type_id);

        let right_type_id = self.eval_expression(*right, context);
        let right_type = context.get_type_from_id(right_type_id);

        let kind = SynTypeKind::Primitive(Primitive::Integer);

        let right_proof = &right_type.proof;

        let mut new_proof = left_type.proof.clone();
        new_proof.search_linear_mut(&mut |left_proof| {
            let mut proofs = Vec::new();

            right_proof.search_linear(&mut |right_proof| {
                match (&left_proof, right_proof) {
                    (Proof::EqualTo(left_proof_type), Proof::EqualTo(right_proof_type)) => {
                        if let (Some(left_literal), Some(right_literal)) =
                            (left_proof_type.literal(), right_proof_type.literal())
                        {
                            let result = operation(left_literal, right_literal).unwrap();

                            proofs.push(Proof::EqualTo(Spanned::new(
                                SynTypeKind::Constant(result),
                                span,
                            )));
                        }
                    }

                    (Proof::EqualTo(left_proof_type), Proof::LessThan(right_proof_type))
                    | (Proof::LessThan(left_proof_type), Proof::EqualTo(right_proof_type))
                    | (Proof::LessThan(left_proof_type), Proof::LessThan(right_proof_type)) => {
                        if let (Some(left_literal), Some(right_literal)) =
                            (left_proof_type.literal(), right_proof_type.literal())
                        {
                            let result = operation(left_literal, right_literal).unwrap();

                            proofs.push(if temp::less_than(left_literal, right_literal).unwrap() {
                                Proof::GreaterThan(Spanned::new(
                                    SynTypeKind::Constant(result),
                                    span,
                                ))
                            } else {
                                Proof::LessThan(Spanned::new(SynTypeKind::Constant(result), span))
                            });
                        }
                    }

                    (Proof::EqualTo(_), Proof::NotEqualTo(_)) => todo!(),
                    (Proof::EqualTo(_), Proof::GreaterThan(_)) => todo!(),
                    (Proof::NotEqualTo(_), Proof::EqualTo(_)) => todo!(),
                    (Proof::NotEqualTo(_), Proof::NotEqualTo(_)) => todo!(),
                    (Proof::NotEqualTo(_), Proof::LessThan(_)) => todo!(),
                    (Proof::NotEqualTo(_), Proof::GreaterThan(_)) => todo!(),
                    (Proof::LessThan(_), Proof::NotEqualTo(_)) => todo!(),
                    (Proof::LessThan(_), Proof::GreaterThan(_)) => todo!(),
                    (Proof::GreaterThan(_), Proof::EqualTo(_)) => todo!(),
                    (Proof::GreaterThan(_), Proof::NotEqualTo(_)) => todo!(),
                    (Proof::GreaterThan(_), Proof::LessThan(_)) => todo!(),
                    (Proof::GreaterThan(_), Proof::GreaterThan(_)) => todo!(),

                    _ => unreachable!(),
                };
            });

            let option = proofs
                .into_iter()
                .reduce(|a, b| Proof::Or(Box::new(a), Box::new(b)));

            if let Some(proof) = option {
                *left_proof = proof;
            }
        });

        context.register_type(SynType::new(new_proof))
    }

    pub fn throw_unknown_at(&mut self, span: SimpleSpan, context: &Context) -> TypeId {
        self.errors
            .push(Spanned::new(Box::from("Unknown type"), span));

        let syn_type = context.register_type(SynType::new(Proof::EqualTo(Spanned::new(
            SynTypeKind::Unknown,
            span,
        ))));

        self.define_type(&syn_type, span, context);

        syn_type
    }

    fn span_with_id(&self, span: SimpleSpan) -> SpanWithModuleId {
        SpanWithModuleId::new(self.module_id, span.into_range())
    }
}

fn nothing_at(span: SimpleSpan, context: &Context) -> TypeId {
    context.register_type(SynType::new(Proof::EqualTo(Spanned::new(
        SynTypeKind::Constant(Literal::Nothing),
        span,
    ))))
}
