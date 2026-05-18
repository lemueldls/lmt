use std::ops::Range;

use facet::Facet;

use crate::ModuleId;

#[repr(u8)]
#[derive(Facet, Debug, Clone, Copy, PartialEq)]
pub enum Span {
    Known {
        start: usize,
        end: usize,
        module_id: ModuleId,
    },
    Unknown,
}

impl Span {
    #[inline]
    #[must_use]
    pub fn new(start: usize, end: usize, module_id: ModuleId) -> Self {
        Self::Known {
            module_id,
            start,
            end,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_optional(range: Option<Range<usize>>, module_id: ModuleId) -> Self {
        range
            .map(|range| Span::new(range.start, range.end, module_id))
            .unwrap_or_else(|| Span::unknown())
    }

    #[inline]
    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    pub fn map<T, F: FnOnce(Range<usize>, ModuleId) -> T>(&self, f: F) -> Option<T> {
        match self {
            Span::Known {
                start,
                end,
                module_id,
            } => Some(f(*start..*end, *module_id)),
            Span::Unknown => None,
        }
    }

    pub fn module_id(&self) -> Option<&ModuleId> {
        match self {
            Span::Known { module_id, .. } => Some(module_id),
            Span::Unknown => None,
        }
    }

    pub fn range(&self) -> Option<Range<usize>> {
        match self {
            Span::Known { start, end, .. } => Some(*start..*end),
            Span::Unknown => None,
        }
    }

    pub fn start(&self) -> Option<usize> {
        match self {
            Span::Known { start, .. } => Some(*start),
            Span::Unknown => None,
        }
    }

    pub fn end(&self) -> Option<usize> {
        match self {
            Span::Known { end, .. } => Some(*end),
            Span::Unknown => None,
        }
    }

    pub fn offset(&self, offset: usize) -> Self {
        match self {
            Span::Known {
                start,
                end,
                module_id,
            } => {
                Span::Known {
                    start: *start + offset,
                    end: *end + offset,
                    module_id: *module_id,
                }
            }
            Span::Unknown => Span::Unknown,
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Span::Unknown)
    }
}
