use std::cmp::max;
use std::{ops::{Index, IndexMut}, slice};
use std::marker::PhantomData;

extern crate proc_macro;

use memory_math::{
    memory_span2d::{MemSpan2D, HasMemSpan2D},
    memory_index2d::MemIndex2D,
    memory_range_iter::IterateWithMemIndex,
    memory_iterators::LinearMemoryIterator,
    memory_offset2d::MemOffset2D,
};
use memory_math::memory_span::MemSpan;
use memory_math::size_2d::{HasSize2D, Size2D};
use super::{vec2d_iter::Vec2DIter};

#[derive(Clone)]
pub struct Vec2D<T> {
    pub size: Size2D,
    items: Vec<T>,
}

impl<T> HasSize2D for Vec2D<T>
{
    fn row_count(&self) -> usize {
        self.size.row_count()
    }

    fn column_count(&self) -> usize {
        self.size.column_count()
    }

    fn size(&self) -> Size2D {
        self.size
    }
}

impl<T> Vec2D<T> {
    pub fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T> {
        self.size.index2d_to_index(coordinates).and_then(|i| self.items.get(i))
    }

    /// Get a slice of a complete row
    fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.row_count() {
            return None;
        }

        let start = row * self.column_count();
        let end = start + self.column_count();
        Some(&self.items[start..end])
    }

    fn get_mut_index2d(&mut self, coordinates: MemIndex2D) -> Option<&mut T> {
        self.size.index2d_to_index(coordinates).and_then(|i| self.items.get_mut(i))
    }
}


impl<T> Index<MemIndex2D> for Vec2D<T> {
    type Output = T;

    fn index(&self, index: MemIndex2D) -> &Self::Output {
        match self.get_index2d(index) {
            Some(val) => val,
            None => panic!(
                "Index2d out of bounds. Index was {} but the size is {}",
                index,
                self.size
            ),
        }
    }
}

impl<T> IndexMut<MemIndex2D> for Vec2D<T> {
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        let extents = self.size;
        match self.get_mut_index2d(index) {
            Some(v) => v,
            None => panic!(
                "Vev2d Coordinates out of bounds. Coordinate was {} but the size is {}",
                index, extents
            ),
        }
    }
}

impl<T> Index<usize> for Vec2D<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<usize> for Vec2D<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.items.get_mut(index) {
            Some(v) => v,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T> Vec2D<T> {

    pub fn new_size_reference(size: Size2D, ref_item: &T) -> Self
    where
        T: Clone,
        Self: Sized,
    {
        let items = vec![ref_item.clone(); size.column_count * size.row_count];
        Vec2D {items, size}
    }

    pub fn from_vec(items: Vec<T>, width: usize) -> Option<Self> {
        if items.len() % width != 0 {
            return None;
        }

        let height = items.len() / width;
        Some(Self {
            size: Size2D::new(height, width),
            items,
        })
    }

    pub fn new_items_rows_columns(items: Vec<T>, rows: usize, columns: usize) -> Option<Self>
    {
        Self::new_items_size(items, Size2D::new(rows, columns))
    }

    pub fn new_items_size(items: Vec<T>, size: Size2D) -> Option<Self> {

        if items.len() != size.column_count * size.row_count {
            return None;
        }

        Some(Vec2D {
            items,
            size
        })
    }


    pub fn push_range(&mut self, start_index: MemIndex2D, range: Vec2D<T>) {
        if start_index.col > self.column_count() || start_index.row > self.row_count() {
            return;
        }

        let shift: MemOffset2D = MemOffset2D::from(start_index);

        for (index, item) in range.linear_iter().iterate_with_mem_index() {
            if let Some(self_index2d) = index + shift{
                self[self_index2d] = item;
            }
        }
    }

    fn get_row_slice(&self, row: usize, span: MemSpan) -> Option<&[T]> {
        let min_col: usize = span.min;
        let max_col: usize = MemSpan::max(&span)?;

        if row >= self.row_count() || max_col >= self.column_count() {
            return None;
        }

        let start = row * self.column_count() + min_col;
        let end = row * self.column_count() + max_col + 1;

        Some(&self.items[start..end])
    }

    /// Get a mutable slice of a complete row
    pub fn get_row_mut(&mut self, row: usize) -> Option<&mut [T]> {
        if row >= self.row_count() {
            return None;
        }

        let start = row * self.column_count();
        let end = start + self.column_count();
        Some(&mut self.items[start..end])
    }

    pub fn get_row_slice_mut(&mut self, row: usize, span: MemSpan) -> Option<&mut [T]> {

        if span.len() == 0 {
            return None;
        }

        if row >= self.row_count() {
            return None;
        }

        let min_col: usize = span.min;
        let max_col: usize = MemSpan::max(&span)?;

        let start = row * self.column_count() + min_col;
        let end = row * self.column_count() + max_col;
        Some(&mut self.items[start..=end])
    }

    /// Get a 2D slice view of a rectangular region
    pub fn get_slice(&self, span2d: MemSpan2D) -> Option<Vec2DSlice<'_, T>>
    {
        if !span2d.valid() || !self.size.contains_span2d(&span2d) {
            return None;
        }

        Some(
            Vec2DSlice::new(
                self.size.column_count,
                span2d,
                &self.items.as_slice()
            )
        )
    }


    pub fn get_slice_mut(&'_ mut self, span2d: MemSpan2D) -> Option<Vec2DMutSlice<'_, T>> {
        if span2d.valid() || !self.size.contains_span2d(&span2d) {
            return None;
        }

        Some(
            Vec2DMutSlice::new(
                self.size.column_count,
                span2d,
                 self.items.as_mut_ptr()
            )
        )
    }

    unsafe fn get_slice_mut_unchecked(&'_ mut self, span2d: MemSpan2D) -> Vec2DMutSlice<'_, T> {
            Vec2DMutSlice::new(
                self.size.column_count,
                span2d,
                self.items.as_mut_ptr()
            )
    }

    pub fn linear_iter(&self) -> Vec2DIter<LinearMemoryIterator, T>
    {
        Vec2DIter::new(LinearMemoryIterator::new(self.size.into()), &self)
    }

    pub fn get_non_overlapping_chunks(&'_ self, spans: Vec<MemSpan2D>) -> Option<Vec<Vec2DSlice<'_, T>>> {

        if MemSpan2D::spans_overlap_or_invalid(&spans)
        {
            return None;
        }

        let mut chunks: Vec<Vec2DSlice<T>> = Vec::with_capacity(spans.len());
        for span in spans
        {
            if let Some(slice) = self.get_slice(span)
            {
                chunks.push(slice);
            }
            else
            {
                return None; //out of bounds
            }
        }

        return Some(chunks);
    }

    pub fn get_non_overlapping_chunks_mut(&'_ mut self, mut spans: Vec<MemSpan2D>) -> Option<Vec<Vec2DMutSlice<'_, T>>> {
        if MemSpan2D::spans_overlap_or_invalid(&spans)
        {
            return None;
        }

        let slice = self.items.as_mut_ptr();
        let len = self.items.len();

        Some(spans.into_iter().map(|s|
        Vec2DMutSlice{
            vec_column_count: self.size.column_count,
            span2d: s,
            data: unsafe{std::slice::from_raw_parts_mut(slice, len).as_mut_ptr()},
            bound: PhantomData
        }).collect())
    }

}

trait SliceMethods
{
    fn vec_column_count(&self) -> usize;

    fn span2d(&self) -> &MemSpan2D;

    fn index2d_to_offset(&self, index2d: MemIndex2D) -> Option<usize>
    {
        if !self.span2d().size().index2d_in_bounds(&index2d)
        {
            return None;
        }

        let vec_index2d: MemIndex2D = self.span2d().relative_index2d_to_absolute_index2d(index2d)?;
        Some(Size2D::index2d_to_index_unchecked(self.vec_column_count(), vec_index2d))
    }

    fn min_offset_for_row(&self, row: usize) -> Option<usize>
    {
        self.index2d_to_offset(MemIndex2D { row, col: 0 })
    }
}

pub struct Vec2DMutSlice<'a, T> {
    pub vec_column_count: usize,
    pub span2d: MemSpan2D,
    data: *mut T,
    bound: PhantomData<&'a T>
}

impl<'a, T> HasSize2D for Vec2DMutSlice<'a, T> {
    fn row_count(&self) -> usize {
        self.span2d.row_count()
    }

    fn column_count(&self) -> usize {
        self.span2d.column_count()
    }
}

impl<'a, T> SliceMethods for Vec2DMutSlice<'a, T>
{
    fn vec_column_count(&self) -> usize
    {
        self.vec_column_count
    }

    fn span2d(&self) -> &MemSpan2D
    {
        &self.span2d
    }
}

impl<'a, T> Index<MemIndex2D> for Vec2DMutSlice<'a, T>
{
    type Output = T;

    fn index(&self, index: MemIndex2D) -> &Self::Output {
        self.get(index).unwrap_or_else(|| panic!("index {} out of bounds", index))
    }
}

impl<'a, T> IndexMut<MemIndex2D> for Vec2DMutSlice<'a, T>
{
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        self.get_mut(index).unwrap_or_else(|| panic!("index {} out of bounds", index))
    }
}

impl<'a, T> Vec2DMutSlice<'a, T> {

    pub fn new(vec_column_count: usize, span2d: MemSpan2D, data: *mut T) -> Vec2DMutSlice<'a, T>
    {
        Vec2DMutSlice
        {
            vec_column_count,
            span2d,
            data,
            bound: PhantomData
        }
    }

    pub fn get(&self, index2d: MemIndex2D) -> Option<&T> {
        let offset: usize = self.index2d_to_offset(index2d)?;
        Some(unsafe {&*self.data.add(offset)})
    }

    pub fn get_mut(&mut self, index2d: MemIndex2D) -> Option<&mut T> {
        let offset: usize = self.index2d_to_offset(index2d)?;
        Some(unsafe {&mut *self.data.add(offset)})
    }
}




/// A 2D slice view that contains a vector of row slices
pub struct Vec2DSlice<'a, T> {
    pub vec_column_count: usize,
    pub span2d: MemSpan2D,
    data: &'a [T]
}

impl<'a, T> SliceMethods for Vec2DSlice<'a, T>
{
    fn vec_column_count(&self) -> usize {
        self.vec_column_count
    }

    fn span2d(&self) -> &MemSpan2D
    {
        &self.span2d
    }
}

impl<'a, T> Vec2DSlice<'a, T> {

    pub fn new(vec_column_count: usize, span2d: MemSpan2D, data: &'a [T]) -> Self {
        Vec2DSlice {
            vec_column_count,
            span2d,
            data
        }
    }

    pub fn get(&self, index2d: MemIndex2D) -> Option<&T> {
        let offset: usize = self.index2d_to_offset(index2d)?;
        self.data.get(offset)
    }

    /// Get a complete row slice within the 2D slice bounds
    pub fn get_span_row(&self, row: usize) -> Option<&[T]> {
        let min_offset: usize = self.min_offset_for_row(row)?;
        let max_offset: usize = min_offset + self.vec_column_count;

        Some(&self.data[min_offset..max_offset])
    }

    /// Iterator over all rows in the 2D slice
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        (0..self.span2d.row_count()).map(move |row| self.get_span_row(row).unwrap())
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Check if the slice contains a value
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.data.contains(x)
    }
}

/// Iterator over elements in a Vec2DSlice

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row() {
        let items: Vec<i32> = (0..16).collect(); // 16 items
        let vec2d = Vec2D::from_vec(items, 4).unwrap(); // 4x4 grid

        // Grid looks like:
        // [ 0,   1,   2,   3]
        // [ 4,   5,   6,   7]
        // [ 8,   9,  10,  11]
        // [12,  13,  14,  15]

        let row = vec2d.get_row(1);

        let expected_row = [4, 5, 6, 7];
        assert_eq!(row, Some(expected_row.as_slice()));
    }

    #[test]
    fn test_get_row_mut() {
        let items: Vec<i32> = (0..16).collect(); // 16 items
        let mut vec2d = Vec2D::from_vec(items, 4).unwrap(); // 4x4 grid

        // Grid looks like:
        // [ 0,   1,   2,   3]
        // [ 4,   5,   6,   7]
        // [ 8,   9,  10,  11]
        // [12,  13,  14,  15]

        let row = vec2d.get_row_mut(1).unwrap();

        let mut expected_row = [4, 5, 6, 7];
        assert_eq!(row, expected_row.as_slice());
    }

    #[test]
    fn test_get_row_slice() {
        let items: Vec<i32> = (0..16).collect(); // 16 items
        let vec2d = Vec2D::from_vec(items, 4).unwrap(); // 4x4 grid

        // Grid looks like:
        // [ 0,   1,   2,   3]
        // [ 4,   5,   6,   7]
        // [ 8,   9,  10,  11]
        // [12,  13,  14,  15]

        let row = vec2d.get_row_slice(1, MemSpan::lower_bound_upper_bound(1, 3).unwrap());
        let expected_row = [5, 6];
        assert_eq!(row, Some(expected_row.as_slice()));
    }

    #[test]
    fn test_get_row_slice_mut() {
        let items: Vec<i32> = (0..16).collect(); // 16 items
        let mut vec2d = Vec2D::from_vec(items, 4).unwrap(); // 4x4 grid

        // Grid looks like:
        // [ 0,   1,   2,   3]
        // [ 4,   5,   6,   7]
        // [ 8,   9,  10,  11]
        // [12,  13,  14,  15]

        let row = vec2d.get_row_slice_mut(1, MemSpan::lower_bound_upper_bound(1, 3).unwrap()).unwrap();
        let expected_row = [5, 6];
        assert_eq!(row, expected_row.as_slice());
    }

    #[test]
    fn test_vec2d_slice() {
        // Create a 4x4 grid with values 0-15
        let items: Vec<i32> = (0..16).collect();
        let vec2d = Vec2D::from_vec(items, 4).unwrap();

        // Grid looks like:
        // [ 0,  1,  2,  3]
        // [ 4,  5,  6,  7]
        // [ 8,  9, 10, 11]
        // [12, 13, 14, 15]

        // Get a 2x2 slice from the center
        let slice = vec2d.get_slice(MemSpan2D::new_from_usize(1, 1, 3, 3)).unwrap();

        assert_eq!(slice.span2d.column_count(), 2);
        assert_eq!(slice.span2d.row_count(), 2);

        // Test element access
        assert_eq!(slice.get_span_index(0, 0), Some(&5));
        assert_eq!(slice.get_span_index(0, 1), Some(&6));
        assert_eq!(slice.get_span_index(1, 0), Some(&9));
        assert_eq!(slice.get_span_index(1, 1), Some(&10));

        // Test row access
        assert_eq!(slice.get_span_row(0), Some([5, 6].as_slice()));
        assert_eq!(slice.get_span_row(1), Some([9, 10].as_slice()));

        // Test iterator
        let values: Vec<i32> = slice.iter().cloned().collect();
        assert_eq!(values, vec![5, 6, 9, 10]);

        // Test conversion to Vec2D
        let new_vec2d = slice.to_vec2d();
        assert_eq!(new_vec2d.get(0, 0), Some(&5));
        assert_eq!(new_vec2d.get(1, 1), Some(&10));
    }

    #[test]
    fn test_vec2d_slice_mut() {
        // Create a 4x4 grid with values 0-15
        let items: Vec<i32> = (0..16).collect();
        let mut vec2d = Vec2D::from_vec(items, 4).unwrap();

        // Grid looks like:
        // [ 0,  1,  2,  3]
        // [ 4,  5,  6,  7]
        // [ 8,  9, 10, 11]
        // [12, 13, 14, 15]

        // Get a 2x2 slice from the center
        let slice = vec2d.get_slice_mut(MemSpan2D::new_from_usize(1, 1, 3, 3)).unwrap();

        assert_eq!(slice.column_count(), 2);
        assert_eq!(slice.row_count(), 2);

        // Test element access
        assert_eq!(slice.get(0, 0), Some(&5));
        assert_eq!(slice.get(0, 1), Some(&6));
        assert_eq!(slice.get(1, 0), Some(&9));
        assert_eq!(slice.get(1, 1), Some(&10));

        // Test row access
        assert_eq!(slice.get_row(0), Some([5, 6].as_slice()));
        assert_eq!(slice.get_row(1), Some([9, 10].as_slice()));

        // Test iterator
        let values: Vec<i32> = slice.iter().cloned().collect();
        assert_eq!(values, vec![5, 6, 9, 10]);

        // Test conversion to Vec2D
        let new_vec2d = slice.to_vec2d();
        assert_eq!(new_vec2d.get(0, 0), Some(&5));
        assert_eq!(new_vec2d.get(1, 1), Some(&10));
    }

    #[test]
    fn test_slice_edge_cases() {
        let items: Vec<i32> = (0..12).collect();
        let vec2d = Vec2D::new_items_rows_columns(items, 4, 3).unwrap(); // 3x4 grid

        //grid looks like:
        // [0, 1, 2]
        // [3, 4, 5]
        // [6, 7, 8]
        // [9,10,11]

        // Single element slice
        let mut slice = vec2d.get_slice(MemSpan2D::new_from_usize(1, 1, 2, 2)).unwrap();
        assert_eq!(slice.span2d.column_count(), 1);
        assert_eq!(slice.span2d.row_count(), 1);
        assert_eq!(slice.get_span_index(0, 0), Some(&4));

        assert_eq!([6,7,8].as_slice(), vec2d.get_row(2).unwrap());

        // Full row slice
        slice = vec2d.get_slice(MemSpan2D::new_from_usize(2, 0, 3, 3)).unwrap();
        assert_eq!(slice.span2d.column_count(), 3);
        assert_eq!(slice.span2d.row_count(), 1);
        assert_eq!(slice.get_span_row(0), Some([6, 7, 8].as_slice()));

        // Invalid bounds should return None
        assert!(vec2d.get_slice(MemSpan2D::new_from_usize(0, 0, 5, 5)).is_none());
        assert!(vec2d.get_slice(MemSpan2D::new_from_usize( 2, 1, 1, 1)).is_none()); // min > max
    }

    #[test]
    fn test_slice_edge_cases_mut() {
        let items: Vec<i32> = (0..12).collect();
        let mut vec2d = Vec2D::new_items_rows_columns(items, 4, 3).unwrap(); // 3x4 grid

        //grid looks like:
        // [0, 1, 2]
        // [3, 4, 5]
        // [6, 7, 8]
        // [9,10,11]

        // Single element slice
        let mut slice = vec2d.get_slice_mut(MemSpan2D::new_from_usize(1, 1, 2, 2)).unwrap();
        assert_eq!(slice.column_count(), 1);
        assert_eq!(slice.row_count(), 1);
        assert_eq!(slice.get(0, 0), Some(&4));

        assert_eq!([6, 7, 8].as_slice(), vec2d.get_row(2).unwrap());

        // Full row slice
        slice = vec2d.get_slice_mut(MemSpan2D::new_from_usize(2, 0, 3, 3)).unwrap();
        assert_eq!(slice.column_count(), 3);
        assert_eq!(slice.row_count(), 1);
        assert_eq!(slice.get_row(0), Some([6, 7, 8].as_slice()));

        // Invalid bounds should return None
        assert!(vec2d.get_slice_mut(MemSpan2D::new_from_usize(0, 0, 5, 5)).is_none());
        assert!(vec2d.get_slice_mut(MemSpan2D::new_from_usize(2, 1, 1, 1)).is_none()); // min > max
    }


    #[test]
    fn test_slice_methods() {
        let items: Vec<i32> = (1..=9).collect();
        let vec2d = Vec2D::from_vec(items, 3).unwrap(); // 3x3 grid with values 1-9

        let slice = vec2d.get_slice(MemSpan2D::new_from_usize(0, 0, 2, 2)).unwrap(); // 2x2 top-left corner

        assert_eq!([1, 2].as_slice(), slice.get_span_row(0).unwrap());
        assert_eq!([4, 5].as_slice(), slice.get_span_row(1).unwrap());
        // Test contains
        assert!(slice.contains(&1));
        assert!(slice.contains(&4));
        assert!(!slice.contains(&9));

        // Test map
        let doubled: Vec<i32> = slice.map(|x| x * 2);
        assert_eq!(doubled, vec![2, 4, 8, 10]);

        // Test rows iterator
        let row_sums: Vec<i32> = slice.rows()
            .map(|row| row.iter().sum())
            .collect();
        assert_eq!(row_sums, vec![3, 9]); // [1+2, 4+5]
    }

    #[test]
    fn test_non_overlapping_chunks_overlap()
    {
        let items: Vec<i32> = (1..=9).collect();
        let vec2d = Vec2D::from_vec(items, 3).unwrap();

        let ranges: Vec<MemSpan2D> = vec![
            {MemSpan2D::new_from_usize(0, 0, 2, 2)},
            {MemSpan2D::new_from_usize(1, 1, 2, 2)}];

        let chunks = vec2d.get_non_overlapping_chunks(ranges);
        assert!(chunks.is_none(), "Expected non, provided spans overlap");
    }

    #[test]
    fn test_non_overlapping_chunks_no_overlap()
    {
        let items: Vec<i32> = (1..=9).collect();
        let vec2d = Vec2D::from_vec(items, 3).unwrap();

        let mut ranges: Vec<MemSpan2D> = vec![
            {MemSpan2D::new_from_usize(0, 0, 1, 2)},
            {MemSpan2D::new_from_usize(2, 0, 3, 2)}];

        let mut chunks = vec2d.get_non_overlapping_chunks(ranges);
        assert!(chunks.is_some(), "Expected some, provided spans do not overlap");
        let mut chunks_unwrap = chunks.unwrap();

        let mut chunk_0 = chunks_unwrap[0].iter().map(|i| i.clone()).collect::<Vec<i32>>();
        let mut chunk_1 = chunks_unwrap[1].iter().map(|i| i.clone()).collect::<Vec<i32>>();

        assert_eq!(chunk_0, vec![1, 2]);
        assert_eq!(chunk_1, vec![7, 8]);

        ranges = vec![
            {MemSpan2D::new_from_usize(0, 0, 1, 3)},
            {MemSpan2D::new_from_usize(1, 0, 2, 3)}];

        chunks = vec2d.get_non_overlapping_chunks(ranges);
        assert!(chunks.is_some(), "Expected some, provided spans do not overlap");

        chunks_unwrap = chunks.unwrap();
        chunk_0 = chunks_unwrap[0].iter().map(|i| i.clone()).collect::<Vec<i32>>();
        chunk_1 = chunks_unwrap[1].iter().map(|i| i.clone()).collect::<Vec<i32>>();

        assert_eq!(chunk_0, vec![1, 2, 3]);
        assert_eq!(chunk_1, vec![4, 5, 6]);
    }

    #[test]
    fn test_mut_non_overlapping_chunks_overlap()
    {
        let items: Vec<i32> = (1..=9).collect();
        let mut vec2d = Vec2D::from_vec(items, 3).unwrap();

        let ranges: Vec<MemSpan2D> = vec![
            {MemSpan2D::new_from_usize(0, 0, 2, 2)},
            {MemSpan2D::new_from_usize(1, 1, 2, 2)}];

        let chunks = vec2d.get_non_overlapping_chunks_mut(ranges);
        assert!(chunks.is_none(), "Expected non, provided spans overlap");
    }

    #[test]
    fn test_mut_non_overlapping_chunks_no_overlap()
    {
        let items: Vec<i32> = (1..=9).collect();
        let mut vec2d = Vec2D::from_vec(items, 3).unwrap();

        let mut ranges: Vec<MemSpan2D> = vec![
            {MemSpan2D::new_from_usize(0, 0, 1, 2)},
            {MemSpan2D::new_from_usize(2, 0, 3, 2)}];

        let mut chunks = vec2d.get_non_overlapping_chunks_mut(ranges);
        assert!(chunks.is_some(), "Expected some, provided spans do not overlap");
        let mut chunks_unwrap = chunks.unwrap();

        let mut chunk_0 = chunks_unwrap[0].iter().map(|i| i.clone()).collect::<Vec<i32>>();
        let mut chunk_1 = chunks_unwrap[1].iter().map(|i| i.clone()).collect::<Vec<i32>>();

        assert_eq!(chunk_0, vec![1, 2]);
        assert_eq!(chunk_1, vec![7, 8]);

        ranges = vec![
            {MemSpan2D::new_from_usize(0, 0, 1, 3)},
            {MemSpan2D::new_from_usize(1, 0, 2, 3)}];

        chunks = vec2d.get_non_overlapping_chunks_mut(ranges);
        assert!(chunks.is_some(), "Expected some, provided spans do not overlap");

        chunks_unwrap = chunks.unwrap();
        chunk_0 = chunks_unwrap[0].iter().map(|i| i.clone()).collect::<Vec<i32>>();
        chunk_1 = chunks_unwrap[1].iter().map(|i| i.clone()).collect::<Vec<i32>>();

        assert_eq!(chunk_0, vec![1, 2, 3]);
        assert_eq!(chunk_1, vec![4, 5, 6]);
    }
}

