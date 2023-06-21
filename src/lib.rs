#![warn(missing_docs)]
//! Iterators adaptors with associated read-only data.
//!
//! Useful for naming the types of wrapped iterators by using function pointers
//! or non-capturing closures.
//!
//! ```
//! use context_iterators::*;
//! use std::ops::Range;
//!
//! type MappedIterator = MapCtx<WithCtx<Range<u16>, u16>, usize>;
//!
//! let iter: MappedIterator = (0..10)
//!     .with_context(42)
//!     .map_with_context(|item: u16, context: &u16| (item + *context) as usize);
//!
//! assert!(iter.eq(42..52));
//! ```
//!
//! The `MappedIterator` type can be used in contexts where a concrete type is
//! needed, for example as an associated type for a trait.
//!
//! ```
//! # use context_iterators::*;
//! # type MappedIterator = MapCtx<WithCtx<std::ops::Range<u16>, u16>, usize>;
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

    /// Get the context.
    fn context_map<F, O>(self, map: F) -> CtxMap<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Context) -> &O,
    {
        CtxMap { iter: self, map }
    }

    /// Apply a map to each element in the iterator.
    fn map_with_context<O>(self, map: fn(Self::Item, &Self::Context) -> O) -> MapCtx<Self, O>
    where
        Self: Sized,
    {
        MapCtx { iter: self, map }
    }

    /// Apply a filter over the elements of the iterator
    fn filter_with_context(self, filter: fn(&Self::Item, &Self::Context) -> bool) -> FilterCtx<Self>
    where
        Self: Sized,
    {
        FilterCtx {
            iter: self,
            predicate: filter,
        }
    }

    /// Apply a filter over the elements of the iterator
    fn filter_map_with_context<O>(
        self,
        filter: fn(Self::Item, &Self::Context) -> Option<O>,
    ) -> FilterMapCtx<Self, O>
    where
        Self: Sized,
    {
        FilterMapCtx {
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

/// Apply a function to the context of an iterator.
#[derive(Clone, Debug)]
pub struct CtxMap<I, F> {
    pub(self) iter: I,
    pub(self) map: F,
}

impl<I, F> Iterator for CtxMap<I, F>
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
        self.iter.size_hint()
    }
}

impl<I, F, O> ContextIterator for CtxMap<I, F>
where
    I: ContextIterator,
    F: Fn(&I::Context) -> &O,
{
    type Context = O;

    fn context(&self) -> &O {
        (self.map)(self.iter.context())
    }
}

impl<I, F> DoubleEndedIterator for CtxMap<I, F>
where
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<I, F> ExactSizeIterator for CtxMap<I, F>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, Ctx> FusedIterator for CtxMap<I, Ctx> where I: FusedIterator {}

/// Map a function over each element in an iterator, passing a context to each
/// function call.
pub type MapWithCtx<I, Ctx, O> = MapCtx<WithCtx<I, Ctx>, O>;

/// Map a function over each element in the iterator.
///
/// Each function call is passed the context of the iterator along with the
/// element.
#[derive(Clone, Debug)]
pub struct MapCtx<I, O>
where
    I: ContextIterator,
{
    pub(self) iter: I,
    pub(self) map: fn(I::Item, &I::Context) -> O,
}

impl<I, O> Iterator for MapCtx<I, O>
where
    I: ContextIterator,
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

impl<I, O> DoubleEndedIterator for MapCtx<I, O>
where
    I: DoubleEndedIterator + ContextIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|item| (self.map)(item, self.iter.context()))
    }
}

impl<I, O> ExactSizeIterator for MapCtx<I, O>
where
    I: ExactSizeIterator + ContextIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, O> FusedIterator for MapCtx<I, O> where I: FusedIterator + ContextIterator {}

impl<I, O> ContextIterator for MapCtx<I, O>
where
    I: ContextIterator,
{
    type Context = I::Context;

    fn context(&self) -> &Self::Context {
        self.iter.context()
    }
}

/// Filter the elements of an iterator, passing a context to each
/// function call.
pub type FilterWithCtx<I, Ctx> = FilterCtx<WithCtx<I, Ctx>>;

/// Filter the elements of an iterator.
///
/// Each function call is passed the context of the iterator along with the
/// element.
#[derive(Clone, Debug)]
pub struct FilterCtx<I>
where
    I: ContextIterator,
{
    pub(self) iter: I,
    pub(self) predicate: fn(&I::Item, &I::Context) -> bool,
}

impl<I> Iterator for FilterCtx<I>
where
    I: ContextIterator,
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

impl<I> DoubleEndedIterator for FilterCtx<I>
where
    I: DoubleEndedIterator + ContextIterator,
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

impl<I> FusedIterator for FilterCtx<I> where I: FusedIterator + ContextIterator {}

impl<I> ContextIterator for FilterCtx<I>
where
    I: ContextIterator,
{
    type Context = I::Context;

    #[inline]
    fn context(&self) -> &Self::Context {
        self.iter.context()
    }
}

/// Map a function over the elements of an iterator, simultaneously filtering elements.
/// Passes a context to each function call.
pub type FilterMapWithCtx<I, Ctx, O> = FilterMapCtx<WithCtx<I, Ctx>, O>;

/// Map a function over the elements of an iterator, simultaneously filtering elements.
///
/// Each function call is passed the context of the iterator along with the
/// element.
#[derive(Clone, Debug)]
pub struct FilterMapCtx<I, O>
where
    I: ContextIterator,
{
    pub(self) iter: I,
    pub(self) predicate: fn(I::Item, &I::Context) -> Option<O>,
}

impl<I, O> Iterator for FilterMapCtx<I, O>
where
    I: ContextIterator,
{
    type Item = O;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Explicit loop to avoid the mutable self.iter borrow while accessing
        // the context.
        loop {
            let item = self.iter.next()?;
            if let Some(elem) = (self.predicate)(item, self.iter.context()) {
                return Some(elem);
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
            sum += (self.predicate)(item, self.iter.context()).is_some() as usize;
        }
        sum
    }
}

impl<I, O> DoubleEndedIterator for FilterMapCtx<I, O>
where
    I: DoubleEndedIterator + ContextIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iter.next_back()?;
            if let Some(elem) = (self.predicate)(item, self.iter.context()) {
                return Some(elem);
            }
        }
    }
}

impl<I, O> FusedIterator for FilterMapCtx<I, O> where I: FusedIterator + ContextIterator {}

impl<I, O> ContextIterator for FilterMapCtx<I, O>
where
    I: ContextIterator,
{
    type Context = I::Context;

    #[inline]
    fn context(&self) -> &Self::Context {
        self.iter.context()
    }
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use super::*;

    #[test]
    fn named_map() {
        type MappedIterator = MapCtx<WithCtx<Range<u16>, u16>, usize>;
        let iter: MappedIterator = (0..10)
            .with_context(42)
            .map_with_context(|item: u16, context: &u16| (item + *context) as usize);

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

    #[test]
    fn filter_map() {
        let iter = (0..10)
            .with_context(42)
            .map_with_context(|item: usize, context: &usize| item + *context);

        assert_eq!(iter.context(), &42);
        assert_eq!(iter.len(), 10);
        assert!(iter.eq(42..52));
    }
}
