use core::{fmt, iter::FusedIterator};

pub trait StatefulMapIter: Iterator {
    fn stateful_map<B, T, F: FnMut(&T, Self::Item) -> (T, B)>(
        self,
        state: T,
        f: F,
    ) -> StatefulMap<Self, T, F>
    where
        Self: Sized,
    {
        StatefulMap::new(self, state, f)
    }
}

impl<I: Iterator> StatefulMapIter for I {}

#[derive(Clone)]
pub struct StatefulMap<I, T, F> {
    iter: I,
    state: T,
    f: F,
}

impl<I, T, F> StatefulMap<I, T, F> {
    pub fn new(iter: I, state: T, f: F) -> StatefulMap<I, T, F> {
        StatefulMap { iter, state, f }
    }

    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I: fmt::Debug, T: fmt::Debug, F> fmt::Debug for StatefulMap<I, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map")
            .field("iter", &self.iter)
            .field("state", &self.state)
            .finish()
    }
}

impl<B, I: Iterator, T, F> Iterator for StatefulMap<I, T, F>
where
    F: FnMut(&T, I::Item) -> (T, B),
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|it| {
            let (state, it) = (&mut self.f)(&self.state, it);

            self.state = state;

            it
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<B, I: DoubleEndedIterator, T, F> DoubleEndedIterator for StatefulMap<I, T, F>
where
    F: FnMut(&T, I::Item) -> (T, B),
{
    #[inline]
    fn next_back(&mut self) -> Option<B> {
        self.iter.next_back().map(|it| {
            let (state, it) = (&mut self.f)(&self.state, it);

            self.state = state;

            it
        })
    }
}

impl<B, I: ExactSizeIterator, T, F> ExactSizeIterator for StatefulMap<I, T, F>
where
    F: FnMut(&T, I::Item) -> (T, B),
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<B, I: FusedIterator, T, F> FusedIterator for StatefulMap<I, T, F> where
    F: FnMut(&T, I::Item) -> (T, B)
{
}
