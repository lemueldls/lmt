use core::{fmt, ops};

use chumsky::{error::Rich, span::SimpleSpan};

pub type Error<'tokens, T> = Rich<'tokens, T, SimpleSpan>;

#[derive(Debug, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct Spanned<T> {
    pub inner: T,
    pub span: SimpleSpan,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: SimpleSpan) -> Self {
        Self { inner, span }
    }

    pub fn span(&self) -> SimpleSpan {
        self.span
    }

    pub fn into_deref(self) -> T {
        self.inner
    }

    pub fn deref_spanned(&self) -> (&T, SimpleSpan) {
        (&self.inner, self.span)
    }

    pub fn into_deref_spanned(self) -> (T, SimpleSpan) {
        (self.inner, self.span)
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned::new(f(self.inner), self.span)
    }

    pub fn span_mut(&mut self) -> &mut SimpleSpan {
        &mut self.span
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
