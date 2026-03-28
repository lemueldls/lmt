pub mod scope;

use std::collections::HashMap;

use lmt_report::SynthesisReport;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use scope::{ScopeId, ScopeMap};

use crate::{SpanWithModuleId, SynType, TypeId, TypeMap, scope::Scope};

#[derive(Debug, Default)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Context {
    pub spanned_proofs: RwLock<HashMap<SpanWithModuleId, TypeId>>,
    pub type_map: TypeMap<SynType>,
    pub scope_map: ScopeMap<Scope>,
    pub reports: RwLock<Vec<SynthesisReport>>,
}

impl Context {
    pub fn register_type(&self, r#type: SynType) -> TypeId {
        self.type_map.push(r#type)
    }

    pub fn get_type_from_id(&self, type_id: TypeId) -> MappedRwLockReadGuard<SynType> {
        self.type_map.get(type_id)
    }

    pub fn get_type_id_from_span(&self, span: SpanWithModuleId) -> TypeId {
        *self.spanned_proofs.read().get(&span).unwrap()
    }

    pub fn register_and_define_type_at_span(&self, r#type: SynType, span: SpanWithModuleId) {
        let type_id = self.register_type(r#type);
        self.define_type_id_at_span(type_id, span);
    }

    pub(crate) fn define_type_id_at_span(&self, type_id: TypeId, span: SpanWithModuleId) {
        self.spanned_proofs.write().insert(span, type_id);
    }
}
