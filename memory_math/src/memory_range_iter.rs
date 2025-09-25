use super::{memory_index2d::MemIndex2D};

pub trait HasCurMemIndex
{
    fn get_cur_mem_index(&self) -> MemIndex2D;
}

pub trait IterateWithMemIndex<T> : Iterator<Item = T> + HasCurMemIndex + Sized
{
    fn iterate_with_mem_index(self) -> IndexEnumerate<Self>
    {
        IndexEnumerate::new(self)
    }
}

impl<T, I: Iterator<Item = T> + HasCurMemIndex> IterateWithMemIndex<T> for I {}

pub struct IndexEnumerate<I>
    where I: HasCurMemIndex + Iterator
{
    iter: I
}

impl<I> Iterator for IndexEnumerate<I>
where I: HasCurMemIndex + Iterator
{
    type Item = (MemIndex2D, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()
        {
            Some(val) => Some((self.iter.get_cur_mem_index(), val)),
            None => None
        }
    }
}

impl<I> IndexEnumerate<I>
    where I: HasCurMemIndex + Iterator
{
    pub fn new(iter: I) -> IndexEnumerate<I> {
        IndexEnumerate { iter }
    }
}
