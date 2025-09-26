use std::fmt::Display;
use crate::memory_index2d::MemIndex2D;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Size2D
{
    pub row_count: usize,
    pub column_count: usize
}

impl Display for Size2D
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size2D({}, {})", self.row_count, self.column_count)
    }
}

impl Size2D
{
    pub fn new(row_count: usize, column_count: usize) -> Self {
        Size2D {
            row_count,
            column_count
        }
    }
}

pub trait HasSize2D
{
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;

    fn max_row(&self) -> Option<usize>
    {
        self.row_count().checked_sub(1)
    }
    
    fn max_col(&self) -> Option<usize>
    {
        self.column_count().checked_sub(1)
    }
    
    #[inline]
    fn size(&self) -> Size2D
    {
        Size2D::new(self.row_count(), self.column_count())
    }
    #[inline]
    fn area(&self) -> usize {
        self.row_count() * self.column_count()
    }

    #[inline]
    fn len(&self) -> usize {
        self.area()
    }

    #[inline]
    fn max_index(&self) -> usize {
        self.area() - 1
    }

    fn index2d_to_index(&self, index2d: MemIndex2D) -> Option<usize> {
        if index2d.row >= self.row_count() || index2d.col >= self.column_count() {
            return None;
        }

        Some(index2d.row * self.column_count() + index2d.col)
    }

    fn index_to_index2d(&self, index: usize) -> Option<MemIndex2D> {
        if index > self.max_index()
        {
            return None;
        }

        Some(MemIndex2D::new(index / self.column_count(), index % self.column_count()))
    }
    
    fn index2d_in_bounds(&self, index2d: &MemIndex2D) -> bool {
        index2d.row < self.row_count() && index2d.col < self.column_count()
    }
}

impl HasSize2D for Size2D
{
    fn row_count(&self) -> usize {
        self.row_count
    }

    fn column_count(&self) -> usize {
        self.column_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index2d_to_index()
    {
        let size = Size2D::new(10, 10);
        let index2d = MemIndex2D::new(5, 5);
        let index = size.index2d_to_index(index2d).unwrap();
        assert_eq!(index, 55);

        assert!(size.index2d_to_index(MemIndex2D::new(10, 10)).is_none());
        assert_eq!(0, size.index2d_to_index(MemIndex2D::origin()).unwrap());
    }
    
    #[test]
    fn test_index_to_index2d()
    {
        let size = Size2D::new(10, 10);
        let index = 55;
        let index2d = size.index_to_index2d(index).unwrap();
        assert_eq!(index2d, MemIndex2D::new(5, 5));

        assert!(size.index_to_index2d(100).is_none());
        assert_eq!(MemIndex2D::origin(), size.index_to_index2d(0).unwrap());
    }
}