use std::ops::Range;

use facet::Facet;

use crate::ModuleId;

#[repr(u8)]
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub const fn new(start: usize, end: usize, module_id: ModuleId) -> Self {
        Self::Known {
            module_id,
            start,
            end,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_optional(range: Option<Range<usize>>, module_id: ModuleId) -> Self {
        range.map_or_else(Self::unknown, |range| Self::new(range.start, range.end, module_id))
    }

    #[inline]
    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    pub fn map<T, F: FnOnce(Range<usize>, ModuleId) -> T>(&self, f: F) -> Option<T> {
        match self {
            Self::Known {
                start,
                end,
                module_id,
            } => Some(f(*start..*end, *module_id)),
            Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn module_id(&self) -> Option<&ModuleId> {
        match self {
            Self::Known { module_id, .. } => Some(module_id),
            Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn range(&self) -> Option<Range<usize>> {
        match self {
            Self::Known { start, end, .. } => Some(*start..*end),
            Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn start(&self) -> Option<usize> {
        match self {
            Self::Known { start, .. } => Some(*start),
            Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn end(&self) -> Option<usize> {
        match self {
            Self::Known { end, .. } => Some(*end),
            Self::Unknown => None,
        }
    }

    #[must_use]
    pub const fn offset(&self, offset: usize) -> Self {
        match self {
            Self::Known {
                start,
                end,
                module_id,
            } => {
                Self::Known {
                    start: *start + offset,
                    end: *end + offset,
                    module_id: *module_id,
                }
            }
            Self::Unknown => Self::Unknown,
        }
    }

    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
