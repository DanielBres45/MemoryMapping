
use std::{fmt::Display, ops::{Add, Sub}};
use std::ops::Range;
use crate::memory_span::MemSpan;
use crate::size_2d::{HasSize2D, Size2D};
use super::{memory_index2d::MemIndex2D, memory_offset2d::MemOffset2D};

#[derive(Clone, Debug)]
pub struct MemSpan2D
{
    pub row_span: MemSpan,
    pub col_span: MemSpan
}

///Represents an index into a 2D span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemSpanIndex2D(pub MemIndex2D);

impl MemSpanIndex2D
{
    pub fn new(row: usize, col: usize) -> Self
    {
        MemSpanIndex2D(MemIndex2D::new(row, col))
    }
}

impl HasSize2D for MemSpan2D
{
    #[inline]
    fn row_count(&self) -> usize {
        self.row_span.len()
    }

    #[inline]
    fn column_count(&self) -> usize {
        self.col_span.len()
    }
}

pub trait HasMemSpan2D
{
    fn column_lower_bound(&self) -> usize;
    fn column_upper_bound(&self) -> usize;
    fn row_lower_bound(&self) -> usize;
    fn row_upper_bound(&self) -> usize;
    #[inline]
    fn extents(&self) -> MemSpan2D {
        MemSpan2D::new_from_index2d(MemIndex2D::new(self.row_lower_bound(), self.column_lower_bound()), MemIndex2D::new(self.row_upper_bound(), self.column_upper_bound()))
    }
}
impl PartialEq for MemSpan2D
{
    fn eq(&self, other: &Self) -> bool {
        self.row_span == other.row_span && self.col_span == other.col_span
    }
}

impl Display for MemSpan2D
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row_span, self.col_span)
    }
}


impl HasMemSpan2D for MemSpan2D
{
    #[inline]
    fn column_lower_bound(&self) -> usize
    {
        self.col_span.min
    }

    #[inline]
    fn column_upper_bound(&self) -> usize
    {
        self.col_span.count
    }

    #[inline]
    fn row_lower_bound(&self) -> usize
    {
        self.row_span.min
    }

    #[inline]
    fn row_upper_bound(&self) -> usize
    {
        self.row_span.count
    }
}

impl Add<MemOffset2D> for MemSpan2D
{
    type Output = Option<MemSpan2D>;

    fn add(self, rhs: MemOffset2D) -> Self::Output {
        let row_range: MemSpan = self.row_span.shift_checked(rhs.row)?;
        let col_range: MemSpan = self.col_span.shift_checked(rhs.col)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }
}

impl Sub<MemOffset2D> for MemSpan2D
{
    type Output = Option<MemSpan2D>;

    fn sub(self, rhs: MemOffset2D) -> Self::Output {
        let flipped = -rhs;
        self + flipped

    }
}


//TODO: Change the nomenclature from vec_index and span_index to absolute_index and relative_index
impl MemSpan2D {

    #[inline]
    pub fn min_column(&self) -> usize
    {
        self.col_span.min
    }

    #[inline]
    pub fn max_column(&self) -> Option<usize>
    {
         MemSpan::max(&self.col_span)
    }

    pub fn max_span_column(&self) -> usize
    {
        self.col_span.len()
    }

    #[inline]
    pub fn min_row(&self) -> usize
    {
        self.row_span.min
    }

    #[inline]
    pub fn contains_row(&self, row: usize) -> bool
    {
        self.row_span.contains(row)
    }

    #[inline]
    pub fn max_row(&self) -> Option<usize>
    {
        MemSpan::max(&self.row_span)
    }

    #[inline]
    pub fn relative_index2d_to_absolute_index2d(&self, index: MemSpanIndex2D) -> Option<MemIndex2D>
    {
        index.0 + MemOffset2D::from(self.min_absolute_index2d())
    }

    #[inline]
    pub fn min_relative_index_for_row(&self, row: usize) -> Option<MemSpanIndex2D>
    {
        if row > self.row_span.max_value()?
        {
            return None;
        }

        Some(MemSpanIndex2D::new(row, 0))
    }

    pub fn min_absolute_index_for_row(&self, row: usize) -> Option<MemIndex2D>
    {
        if row < self.min_row() || row > self.max_row()?
        {
            return None;
        }

        let min_col: usize = self.min_column();
        Some(MemIndex2D::new(row, min_col))
    }

    #[inline]
    pub fn max_relative_index_for_row(&self, row: usize) -> Option<MemSpanIndex2D>
    {
        let max_col: usize = self.size().max_col()?;

        if max_col == 0
        {
            return None;
        }

        Some(MemSpanIndex2D::new(row, max_col))
    }

    pub fn max_absolute_index_for_row(&self, row: usize) -> Option<MemIndex2D>
    {
        if row < self.min_row() || row > self.max_row()?
        {
            return None;
        }

        let max_col: usize = self.max_column()?;
        Some(MemIndex2D::new(row, max_col))
    }

    #[inline]
    pub fn min_relative_index2d(&self) -> MemSpanIndex2D
    {
        MemSpanIndex2D(MemIndex2D::origin())
    }

    #[inline]
    pub fn max_relative_index2d(&self) -> Option<MemSpanIndex2D>
    {
        match self.size().max_index2d()
        {
            Some(max_index2d) => Some(MemSpanIndex2D(max_index2d)),
            None => None
        }
    }

    pub fn min_absolute_index2d(&self) -> MemIndex2D
    {
        let min_row = self.min_row();
        let min_col = self.min_column();
        MemIndex2D::new(min_row, min_col)
    }

    pub fn max_absolute_index2d(&self) -> Option<MemIndex2D>
    {
        let max_row: usize = self.max_row()?;
        let max_col: usize = self.max_column()?;
        Some(MemIndex2D::new(max_row, max_col))
    }

    #[inline]
    pub fn validate_coordinates(min_coord: MemIndex2D, max_coord: MemIndex2D) -> bool
    {
        MemOffset2D::from(min_coord) <= MemOffset2D::from(max_coord)
    }

    pub fn new_row_columns(rows: usize, columns: usize) -> Self
    {
        MemSpan2D
        {
            row_span: MemSpan::new_range(0..rows),
            col_span: (0..columns).into()
        }
    }

    pub fn new_from_index2d(min_index2d: MemIndex2D, upper_bound_index2d: MemIndex2D) -> Self
    {
        MemSpan2D
        {
            row_span: MemSpan::new_range(min_index2d.row..upper_bound_index2d.row),
            col_span: MemSpan::new_range(min_index2d.col..upper_bound_index2d.col)
        }
    }

    pub fn area(&self) -> usize
    {
        self.row_count() * self.column_count()
    }

    pub fn new_from_usize(row_lower_bound: usize, column_lower_bound: usize, row_upper_bound: usize, column_upper_bound: usize) -> Self
    {
        MemSpan2D
        {
            row_span: MemSpan::new_range(row_lower_bound..row_upper_bound),
            col_span: MemSpan::new_range(column_lower_bound..column_upper_bound)
        }
    }

    pub fn shift_rows(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.shift_checked(shift)?;
        let col_range: MemSpan = self.col_span.clone();

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift_columns(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.clone();
        let col_range: MemSpan = self.col_span.shift_checked(shift)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift(&self, shift: MemOffset2D) -> Option<Self> {
        let row_range: MemSpan = self.row_span.shift_checked(shift.row)?;
        let col_range: MemSpan = self.col_span.shift_checked(shift.col)?;
        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift_max_rows(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.shift_max_checked(shift)?;
        let col_range: MemSpan = self.col_span.clone();

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift_max_columns(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.clone();
        let col_range: MemSpan = self.col_span.shift_max_checked(shift)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift_max(&self, shift: MemOffset2D) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.shift_max_checked(shift.row)?;
        let col_range: MemSpan = self.col_span.shift_max_checked(shift.col)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn shift_min_rows(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.shift_min_checked(shift)?;
        let col_range: MemSpan = self.col_span.clone();

        Some(
            MemSpan2D
            {
                row_span: row_range,
                col_span: col_range
            }
        )
    }

    pub fn shift_min_columns(&self, shift: isize) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.clone();
        let col_range: MemSpan = self.col_span.shift_min_checked(shift)?;

        Some(
            MemSpan2D
            {
                row_span: row_range,
                col_span: col_range
            }
        )
    }

    pub fn shift_min(&self, shift: MemOffset2D) -> Option<Self>
    {
        let row_range: MemSpan = self.row_span.shift_min_checked(shift.row)?;
        let col_range: MemSpan = self.col_span.shift_min_checked(shift.col)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn contains(&self, index2d: &MemSpanIndex2D) -> bool
    {
        self.row_span.contains(index2d.0.row) && self.col_span.contains(index2d.0.col)
    }

    pub fn intersect(&self, other: &MemSpan2D) -> Option<MemSpan2D>
    {
        let row_range = self.row_span.intersect(&other.row_span)?;
        let col_range = self.col_span.intersect(&other.col_span)?;

        Some(MemSpan2D
        {
            row_span: row_range,
            col_span: col_range
        })
    }

    pub fn overlaps(&self, other: &MemSpan2D) -> bool
    {
        self.intersect(other).is_some_and(|s| s.area() > 0 )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing()
    {
        let span: MemSpan2D = MemSpan2D::new_from_usize(1,1,3,3);

        let min_row_span_index2d: MemSpanIndex2D = span.min_relative_index_for_row(0).unwrap();
        assert_eq!(MemIndex2D::new(0,0), min_row_span_index2d.0);

        let min_row_index2d: MemIndex2D = span.relative_index2d_to_absolute_index2d(min_row_span_index2d).unwrap();
        assert_eq!(MemIndex2D::new(1,1), min_row_index2d);

    }

    #[test]
    fn test_intersect_col_disjoint()
    {
        let mut left = MemSpan2D::new_from_index2d(MemIndex2D::origin(), MemIndex2D::new(10, 10));
        let mut right = MemSpan2D::new_from_index2d(MemIndex2D::new(0, 11), MemIndex2D::new(10, 21));

        assert!(!left.overlaps(&right));

        left = MemSpan2D::new_from_index2d(MemIndex2D::origin(), MemIndex2D::new(10, 10));
        right = MemSpan2D::new_from_index2d(MemIndex2D::new(0, 10), MemIndex2D::new(10, 20));

        assert!(!left.overlaps(&right));

    }

    #[test]
    fn test_intersect_col_lower_upper_overlap()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 10);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(0, 10, 10, 20);
        let expected_intersection: MemSpan2D = MemSpan2D::new_from_usize(0, 10, 10, 10);
        assert_eq!(lhs.intersect(&rhs).unwrap(), expected_intersection);
        assert_eq!(rhs.intersect(&lhs).unwrap(), expected_intersection);
        assert_eq!(0, expected_intersection.area());
    }

    #[test]
    fn test_intersect_col_overlap()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 10);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(0, 5, 10, 15);
        let expected_intersection: MemSpan2D = MemSpan2D::new_from_usize(0, 5, 10, 10);
        assert_eq!(lhs.intersect(&rhs).unwrap(), expected_intersection);
        assert_eq!(rhs.intersect(&lhs).unwrap(), expected_intersection);
        assert_eq!(50, expected_intersection.area());
    }

    #[test]
    fn test_row_lower_upper_overlap()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 10);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(10, 0, 20, 10);
        let expected_intersection: MemSpan2D = MemSpan2D::new_from_usize(10, 0, 10, 10);
        assert_eq!(lhs.intersect(&rhs).unwrap(), expected_intersection);
        assert_eq!(rhs.intersect(&lhs).unwrap(), expected_intersection);
        assert_eq!(0, expected_intersection.area());
    }

    #[test]
    fn test_intersect_row_disjoint()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 10);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(11, 0, 21, 10);

        assert!(lhs.intersect(&rhs).is_none());
        assert!(rhs.intersect(&lhs).is_none());
    }

    #[test]
    fn test_intersect_row_overlap()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 20);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(5, 10, 15, 20);
        let expected_intersection: MemSpan2D = MemSpan2D::new_from_usize(5, 10, 10, 20);
        assert_eq!(lhs.intersect(&rhs).unwrap(), expected_intersection);
        assert_eq!(rhs.intersect(&lhs).unwrap(), expected_intersection);
        assert_eq!(50, expected_intersection.area());
    }

    #[test]
    fn test_intersect_inside()
    {
        let lhs: MemSpan2D = MemSpan2D::new_from_usize(0, 0, 10, 10);
        let rhs: MemSpan2D = MemSpan2D::new_from_usize(5, 5, 9, 9);
        let expected_intersection: MemSpan2D = MemSpan2D::new_from_usize(5, 5, 9, 9);
        assert_eq!(lhs.intersect(&rhs).unwrap(), expected_intersection);
        assert_eq!(rhs.intersect(&lhs).unwrap(), expected_intersection);
    }
}