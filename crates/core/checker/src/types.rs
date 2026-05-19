use facet::Facet;
use lmt_solver::SmtExpr;

#[repr(u8)]
#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Real,
    Bool,
    String,
    Refinement {
        base: Box<Type>,
        binder: String,
        predicate: SmtExpr,
    },
    Arrow {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    Error,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
#[facet(transparent)]
pub struct Env(pub Vec<(String, Type)>);

impl Env {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, name: &str) -> Option<Type> {
        self.0
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }

    pub fn extend(&self, name: String, ty: Type) -> Self {
        let mut new_env = self.0.clone();
        new_env.push((name, ty));

        Self(new_env)
    }
}
