use std::fmt::Display;
use crate::memory_index2d::MemIndex2D;
use crate::memory_span2d::MemSpan2D;

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

impl Into<MemSpan2D> for Size2D
{
    fn into(self) -> MemSpan2D
    {
        MemSpan2D::new_row_columns(self.row_count, self.column_count)
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

    #[inline]
    pub fn area(&self) -> usize {
        self.row_count() * self.column_count()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.area()
    }

    #[inline]
    pub fn max_index(&self) -> usize {
        self.area() - 1
    }

    #[inline]
    pub fn max_index2d(&self) -> Option<MemIndex2D>
    {
        let max_row: usize = self.max_row()?;
        let max_col: usize = self.max_col()?;
        Some(MemIndex2D::new(max_row, max_col))
    }

    pub fn index2d_to_index(&self, index2d: MemIndex2D) -> Option<usize> {
        if index2d.row >= self.row_count() || index2d.col >= self.column_count() {
            return None;
        }

        Some(Size2D::index2d_to_index_unchecked(self.column_count, index2d))
    }

    #[inline]
    pub fn index2d_to_index_unchecked(column_count: usize, index2d: MemIndex2D) -> usize {
        index2d.row * column_count + index2d.col
    }


    pub fn row_column_to_index(&self, row: usize, column: usize) -> Option<usize>
    {
        self.index2d_to_index(MemIndex2D::new(row, column))
    }

    pub fn index_to_index2d(&self, index: usize) -> Option<MemIndex2D> {
        if index > self.max_index()
        {
            return None;
        }

        Some(MemIndex2D::new(index / self.column_count(), index % self.column_count()))
    }

    pub fn index2d_in_bounds(&self, index2d: &MemIndex2D) -> bool {
        index2d.row < self.row_count() && index2d.col < self.column_count()
    }

    pub fn index2d_in_bounds_exclusive(&self, index2d: &MemIndex2D) -> bool {
        index2d.row <= self.row_count() && index2d.col <= self.column_count()
    }

    pub fn contains_span2d(&self, span2d: &MemSpan2D) -> bool {
        self.index2d_in_bounds_exclusive(&span2d.min_absolute_index2d())
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