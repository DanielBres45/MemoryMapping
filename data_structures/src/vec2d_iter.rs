use std::{usize, vec::IntoIter};
use std::marker::PhantomData;
use std::ops::Index;
use memory_math::{memory_index2d::MemIndex2D, memory_range_iter::HasCurMemIndex};
use memory_math::memory_iterators::{LinearMemoryIterator, MemoryIterator};
use crate::vec2d::{Vec2D};

trait Vec2DIterator<'a, I: MemoryIterator, T>
{
    fn span_iterator(&mut self) -> &mut I;

    fn get(&self, index2d: MemIndex2D) -> Option<&'a T>;

    fn next_item(&mut self) -> Option<&'a T>
    {
        self.span_iterator().next().and_then(|i| self.get(i))
    }
}

pub struct Vec2DIntoIter<I, T>
    where
    I: MemoryIterator
{
    iter: I,
    items: Vec2D<T>,
}

impl<I: MemoryIterator, T> Iterator for Vec2DIntoIter<I, T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|i| Some(&self.items[i] ))
    }
}

