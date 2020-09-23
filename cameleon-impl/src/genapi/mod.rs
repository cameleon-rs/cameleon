mod elem_type;
mod node_base;
mod verifier;
mod xml;

pub use node_base::*;

use std::ops::{Deref, Range};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenApiError {
    #[error("xml error: {}", 0)]
    XmlError(#[from] roxmltree::Error),

    #[error("required field missing: {}", Deref::deref(.0))]
    RequiredFieldMissing(Span<&'static str>),

    #[error("invalid data: {}", Deref::deref(.0))]
    InvalidData(Span<String>),

    #[error("<{}> has no text", Deref::deref(.0))]
    ElementIsEmpty(Span<String>),
}

pub type GenApiResult<T> = std::result::Result<T, GenApiError>;

pub struct Span<T> {
    inner: T,
    start: usize,
    end: usize,
}

impl<T> Span<T> {
    pub fn new(inner: T, start: usize, end: usize) -> Self {
        Self { inner, start, end }
    }

    pub fn span<U>(&self, inner: U) -> Span<U> {
        Span::from_range(inner, self.range())
    }

    pub fn from_range(inner: T, range: Range<usize>) -> Self {
        Self::new(inner, range.start, range.end)
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn map<U, F>(self, f: F) -> Span<U>
    where
        F: FnOnce(T) -> U,
    {
        let (start, end) = (self.start, self.end);
        let res = f(self.into_inner());
        Span::new(res, start, end)
    }
}

impl<T, U> Span<Result<T, U>> {
    pub fn transpose(self) -> Result<Span<T>, U> {
        match self.inner {
            Ok(inner) => {
                let (start, end) = (self.start, self.end);
                Ok(Span::new(inner, start, end))
            }
            Err(e) => Err(e),
        }
    }
}

impl<T> std::ops::Deref for Span<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Span<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Clone> Clone for Span<T> {
    fn clone(&self) -> Span<T> {
        Self {
            inner: self.inner.clone(),
            start: self.start,
            end: self.end,
        }
    }
}

impl<T: Copy> Copy for Span<T> {}

impl<T: std::fmt::Debug> std::fmt::Debug for Span<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("inner", &self.inner)
            .finish()
    }
}
