#![warn(missing_docs)]
//! Iterators adaptors with associated read-only data.
//!
//! Useful for naming the types of wrapped iterators by using function pointers
//! or non-capturing closures.
//!
//! ```
//! use context_iterators::*;
//!
//! type Closure = fn(usize, &usize) -> usize;
//! type MappedIterator = MapCtx<WithCtx<std::ops::Range<usize>, usize>, Closure>;
//!
//! let iter: MappedIterator = (0..10)
//!     .with_context(42)
//!     .map_with_context(|item: usize, context: &usize| item + *context);
//!
//! assert!(iter.eq(42..52));
//! ```
//!
//! The `MappedIterator` type can be used in contexts where a concrete type is
//! needed, for example as an associated type for a trait.
//!
//! ```
//! # use context_iterators::*;
//! # type Closure = fn(usize, &usize) -> usize;
//! # type MappedIterator = MapCtx<WithCtx<std::ops::Range<usize>, usize>, Closure>;
//! trait Iterable {
//!     type Iter: Iterator<Item = usize>;
//! }
//!
//! struct MyIterable;
//!
//! impl Iterable for MyIterable {
//!    type Iter = MappedIterator;
//! }
//! ```

use std::iter::FusedIterator;

/// Extended iterator trait to allow adding context data.
///
/// This trait is automatically implemented for all iterators.
pub trait IntoContextIterator: Iterator {
    /// Add read-only context to the iterator.
    fn with_context<Ctx>(self, context: Ctx) -> WithCtx<Self, Ctx>
    where
        Self: Sized,
    {
        WithCtx {
            iter: self,
            context,
        }
    }
}

impl<I> IntoContextIterator for I where I: Iterator {}

/// Iterator carrying a context.
pub trait ContextIterator: Iterator {
    /// The context type.
    type Context;

    /// Get the context.
    fn context(&self) -> &Self::Context;

    /// Apply a map to each element in the iterator.
    fn map_with_context<F>(self, map: F) -> MapCtx<Self, F>
    where
        Self: Sized,
    {
        MapCtx { iter: self, map }
    }

    /// Apply a filter over the elements of the iterator
    fn filter_with_context<F>(self, filter: F) -> FilterCtx<Self, F>
    where
        Self: Sized,
    {
        FilterCtx {
            iter: self,
            predicate: filter,
        }
    }
}

/// Wrapper around an iterator adding context data.
#[derive(Clone, Debug)]
pub struct WithCtx<I, Ctx> {
    pub(self) iter: I,
    pub(self) context: Ctx,
}

impl<I, Ctx> Iterator for WithCtx<I, Ctx>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, Ctx> ContextIterator for WithCtx<I, Ctx>
where
    I: Iterator,
{
    type Context = Ctx;

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

impl<I, Ctx> DoubleEndedIterator for WithCtx<I, Ctx>
where
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<I, Ctx> ExactSizeIterator for WithCtx<I, Ctx>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, Ctx> FusedIterator for WithCtx<I, Ctx> where I: FusedIterator {}

/// Map a function over each element in the iterator.
#[derive(Clone, Debug)]
pub struct MapCtx<I, F> {
    pub(self) iter: I,
    pub(self) map: F,
}

impl<I, F, O> Iterator for MapCtx<I, F>
where
    I: ContextIterator,
    F: FnMut(I::Item, &I::Context) -> O,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|item| (self.map)(item, self.iter.context()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, F, O> DoubleEndedIterator for MapCtx<I, F>
where
    I: DoubleEndedIterator + ContextIterator,
    F: FnMut(I::Item, &I::Context) -> O,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|item| (self.map)(item, self.iter.context()))
    }
}

impl<I, F, O> ExactSizeIterator for MapCtx<I, F>
where
    I: ExactSizeIterator + ContextIterator,
    F: FnMut(I::Item, &I::Context) -> O,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, F, O> FusedIterator for MapCtx<I, F>
where
    I: FusedIterator + ContextIterator,
    F: FnMut(I::Item, &I::Context) -> O,
{
}

impl<I, F, O> ContextIterator for MapCtx<I, F>
where
    I: ContextIterator,
    F: FnMut(I::Item, &I::Context) -> O,
{
    type Context = I::Context;

    fn context(&self) -> &Self::Context {
        self.iter.context()
    }
}

/// Map a function over an iterator, filtering
#[derive(Clone, Debug)]
pub struct FilterCtx<I, F> {
    pub(self) iter: I,
    pub(self) predicate: F,
}

impl<I, F> Iterator for FilterCtx<I, F>
where
    I: ContextIterator,
    F: FnMut(&I::Item, &I::Context) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Explicit loop to avoid the mutable self.iter borrow while accessing
        // the context.
        loop {
            let item = self.iter.next()?;
            if (self.predicate)(&item, self.iter.context()) {
                return Some(item);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }

    #[inline]
    fn count(mut self) -> usize {
        let mut sum = 0;
        while let Some(item) = self.iter.next() {
            sum += (self.predicate)(&item, self.iter.context()) as usize;
        }
        sum
    }
}

impl<I, F> DoubleEndedIterator for FilterCtx<I, F>
where
    I: DoubleEndedIterator + ContextIterator,
    F: FnMut(&I::Item, &I::Context) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iter.next_back()?;
            if (self.predicate)(&item, self.iter.context()) {
                return Some(item);
            }
        }
    }
}

impl<I, F> FusedIterator for FilterCtx<I, F>
where
    I: FusedIterator + ContextIterator,
    F: FnMut(&I::Item, &I::Context) -> bool,
{
}

impl<I, F> ContextIterator for FilterCtx<I, F>
where
    I: ContextIterator,
    F: FnMut(&I::Item, &I::Context) -> bool,
{
    type Context = I::Context;

    #[inline]
    fn context(&self) -> &Self::Context {
        self.iter.context()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn named_map() {
        type Closure = fn(usize, &usize) -> usize;
        type MappedIterator = MapCtx<WithCtx<std::ops::Range<usize>, usize>, Closure>;

        let iter: MappedIterator = (0..10)
            .with_context(42)
            .map_with_context(|item: usize, context: &usize| item + *context);

        assert_eq!(iter.context(), &42);
        assert_eq!(iter.len(), 10);
        assert!(iter.eq(42..52));
    }

    #[test]
    fn filter() {
        let iter = (0..10)
            .with_context(42)
            .filter_with_context(|item: &usize, context: &usize| item + *context >= 50);

        assert_eq!(iter.context(), &42);
        assert_eq!(iter.clone().count(), 2);
        assert!(iter.eq(8..10));
    }
}
