use std::{self, fmt, marker::PhantomData};

use hashbrown::{
    HashMap,
    hash_map::{Entry, OccupiedError},
};
use lmt_parser::Spanned;
use lmt_report::{SynthesisReport, miette::SourceSpan};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};

use crate::{TypeId, define_index_type, util::SlotLockMap};

#[derive(Debug, Default)]
pub struct Scope {
    idents: HashMap<String, Spanned<TypeId>>,
    extends: Option<ScopeId>,
}

impl Scope {
    pub fn define(
        &mut self,
        ident_spanned: Spanned<String>,
        type_id: TypeId,
    ) -> lmt_report::Result<()> {
        let (ident, span) = ident_spanned.into_deref_spanned();

        if let Some(previous) = self
            .idents
            .insert(ident.clone(), Spanned::new(type_id, span))
        {
            Err(SynthesisReport::AlreadyDefined {
                ident,
                redefined_span: SourceSpan::from(span.into_range()),
                originally_defined_span: SourceSpan::from(previous.span().into_range()),
            })
        } else {
            Ok(())
        }
    }

    pub fn get_ident(&self) {}
}

define_index_type! { pub ScopeId }
pub type ScopeMap<T> = SlotLockMap<ScopeId, T>;
