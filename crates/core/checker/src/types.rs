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
        base: Box<Self>,
        binder: String,
        predicate: SmtExpr,
    },
    Arrow {
        params: Vec<Self>,
        ret: Box<Self>,
    },
    Error,
}

#[derive(Facet, Debug, Clone, PartialEq, Eq, Hash)]
#[facet(transparent)]
pub struct Env(pub Vec<(String, Type)>);

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Type> {
        self.0
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }

    #[must_use]
    pub fn extend(&self, name: String, ty: Type) -> Self {
        let mut new_env = self.0.clone();
        new_env.push((name, ty));

        Self(new_env)
    }
}
