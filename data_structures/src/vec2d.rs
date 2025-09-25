use std::cmp::max;
use std::{ops::{Index, IndexMut}, slice};
use std::marker::PhantomData;

extern crate proc_macro;
use proc_macro::TokenStream;

use memory_math::{
    memory_span2d::{MemSpan2D, HasMemSpan2D},
    memory_index2d::MemIndex2D,
    memory_range_iter::IterateWithMemIndex,
    memory_iterators::LinearMemoryIterator,
    memory_offset2d::MemOffset2D,
};
use memory_math::memory_span::MemSpan;
use memory_math::size_2d::{HasSize2D, Size2D};
use super::{vec2d_iter::Vec2DIntoIter};

pub trait Vec2DMethods<T> : HasSize2D
{
    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T>;

    /// Get a reference to an element at 2D coordinates
    fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.get_index2d(MemIndex2D::new(row, col))
    }
    fn get_row(&self, row: usize) -> Option<&[T]>;

}

pub trait MutVec2DMethods<T> : Vec2DMethods<T>
{
    fn get_mut_index2d(&mut self, coordinates: MemIndex2D) -> Option<&mut T>;

    /// Get a mutable reference to an element at 2D coordinates
    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut T> {
        self.get_mut_index2d(MemIndex2D::new(row, col))
    }
}
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

impl<T> Vec2DMethods<T> for Vec2D<T> {
    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T> {
        self.index2d_to_index(coordinates).and_then(|i| self.items.get(i))
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

}

impl<T> MutVec2DMethods<T> for Vec2D<T> {
    fn get_mut_index2d(&mut self, coordinates: MemIndex2D) -> Option<&mut T> {
        self.index2d_to_index(coordinates).and_then(|i| self.items.get_mut(i))
    }
}

impl<T> IntoIterator for Vec2D<T> {
    type Item = T;
    type IntoIter = Vec2DIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {

        let column_count = self.column_count();
        Vec2DIntoIter::new(self.items.into_iter(), column_count)
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

        for (index, item) in range.into_iter().iterate_with_mem_index() {
            if let Some(self_index2d) = index + shift{
                self[self_index2d] = item;
            }
        }
    }

    fn get_row_slice(&self, row: usize, span: MemSpan) -> Option<&[T]> {
        let min_col: usize = span.min;
        let max_col: usize = MemSpan::max(&span);

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
        let max_col: usize = MemSpan::max(&span);

        let start = row * self.column_count() + min_col;
        let end = row * self.column_count() + max_col;
        Some(&mut self.items[start..=end])
    }

    /// Get a 1D slice view of a rectangular region
    pub fn get_slice(&self, span2d: MemSpan2D) -> Option<Vec2DSlice<T>> {

        let min_row: usize = span2d.min_row();
        let max_row: usize = span2d.max_row();

        if max_row >= self.row_count() {
            return None;
        }

        let mut row_slices = Vec::with_capacity(max_row - min_row + 1);
        for row in min_row..=max_row {
            if let Some(row_slice) = self.get_row_slice(row, span2d.col_span) {
                row_slices.push(row_slice);
            } else {
                return None;
            }
        }

        let size: Size2D = Size2D::new(span2d.row_span.len(), span2d.column_count());

        Some(Vec2DSlice {
            row_slices,
            size
        })
    }


    //TODO: Can we make an unsafe variant of this, to allow the non overlapping chunks mut to use the same pointer logic...
    pub fn get_slice_mut(&'_ mut self, span2d: MemSpan2D) -> Option<Vec2DMutSlice<'_, T>> {
        if span2d.area() == 0 {
            return None;
        }

        let min_index2d: MemIndex2D = span2d.min_index2d();
        let max_index2d: MemIndex2D = span2d.max_index2d();

        let min_index: usize = self.index2d_to_index(min_index2d)?;
        let max_index: usize = self.index2d_to_index(max_index2d)?;

        let mut slice  = self.items[min_index..=max_index].as_mut_ptr();
        let mut row_slices: Vec<&mut [T]> = Vec::with_capacity(max_index2d.row - min_index2d.row + 1);
        let col_length: usize = span2d.col_span.len();

        for row in span2d.row_span.into_iter() {
            if row != min_index2d.row { unsafe { slice = slice.add(min_index2d.col); } } //shift to start of row slice

            let cur_slice: &mut [T] = unsafe {slice::from_raw_parts_mut(slice, col_length)};
        row_slices.push(cur_slice);

            if row != max_index2d.row { unsafe { slice = slice.add(self.column_count() - max_index2d.col + 1); } } //shift to end of row
        }

        let size: Size2D = Size2D::new(span2d.row_span.len(), span2d.column_count());
        Some(Vec2DMutSlice
        {
            rows: row_slices,
            size
        })
    }

    fn spans_overlap_or_invalid(spans: &Vec<MemSpan2D>) -> bool
    {
        for i in 0..spans.len()
        {
            if spans[i].area() == 0
            {
                return true;
            }

            for j in 0..spans.len()
            {
                if i == j
                {
                    continue;
                }

                if spans[i].overlaps(&spans[j])
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_non_overlapping_chunks(&'_ self, spans: Vec<MemSpan2D>) -> Option<Vec<Vec2DSlice<'_, T>>> {

        if Self::spans_overlap_or_invalid(&spans)
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
        if Self::spans_overlap_or_invalid(&spans)
        {
            return None;
        }
        spans.sort_by_key(|span| span.min_index2d());

        // Validate all spans are within bounds
        for span in &spans {
            let min_idx = span.min_index2d();
            let max_idx = span.max_index2d();
            if !self.index2d_in_bounds(&min_idx) || !self.index2d_in_bounds(&max_idx) {
                return None;
            }
        }

        // Use raw pointers to create non-overlapping slices
        let base_ptr = self.items.as_mut_ptr();
        let mut result = Vec::with_capacity(spans.len());

        for span in &spans {
            let mut rows = Vec::with_capacity(span.row_span.len());

            for row in span.row_span.clone() {
                let start_idx = self.index2d_to_index(MemIndex2D { row, col: span.col_span.min })?;
                let len = span.col_span.len();

                unsafe {
                    let slice_ptr = base_ptr.add(start_idx);
                    let slice = std::slice::from_raw_parts_mut(slice_ptr, len);
                    rows.push(slice);
                }
            }

            let size: Size2D = Size2D::new(span.row_span.len(), span.column_count());
            result.push(Vec2DMutSlice {
                rows,
                size
            });
        }

        Some(result)
    }

}

pub struct Vec2DMutSlice<'a, T> {
    rows: Vec<&'a mut [T]>,
    pub size: Size2D
}

impl<'a, T> HasSize2D for Vec2DMutSlice<'a, T> {
    fn row_count(&self) -> usize {
        self.size.row_count
    }

    fn column_count(&self) -> usize {
        self.size.column_count
    }

    fn size(&self) -> Size2D {
        self.size
    }
}


impl<'a, T> Vec2DMethods<T> for Vec2DMutSlice<'a, T> {

    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T> {
        self.get(coordinates.row, coordinates.col)
    }

    fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.row_count() {
            return None;
        }

        Some(&*self.rows[row])
    }
}

impl<'a, T> MutVec2DMethods<T> for Vec2DMutSlice<'a, T> {
    fn get_mut_index2d(&mut self, coordinates: MemIndex2D) -> Option<&mut T> {
        self.get_mut(coordinates.row, coordinates.col)
    }
}

impl<'a, T> Vec2DMutSlice<'a, T> {

    unsafe fn get_row_unchecked(&self, row: usize) -> &[T] {
        &*self.rows[row]
    }

    /// Get a reference to an element at slice-relative coordinates
    /// # Safety
    /// Caller must ensure that row and col are within bounds
    unsafe fn get_unchecked(&self, row: usize, col: usize) -> &T {
        let row_ptr = self.rows.get_unchecked(row);
        &*(*row_ptr).as_ptr().add(col)
    }

    // /// Get a mutable reference to an element at slice-relative coordinates
    // /// # Safety
    // /// Caller must ensure that row and col are within bounds and that no other
    // /// mutable references to the same element exist
    // unsafe fn get_unchecked_mut(&self, row: usize, col: usize) -> &mut T {
    //     let row_ptr = self.rows.get_unchecked(row);
    //     &mut *(*row_ptr).as_mut_ptr().add(col)
    // }

    /// Get a reference to an element at slice-relative coordinates (bounds checked)
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.row_count() || col >= self.column_count() {
            return None;
        }
        Some(unsafe { self.get_unchecked(row, col) })
    }

    // /// Get a mutable reference to an element at slice-relative coordinates (bounds checked)
    // pub fn get_mut(&self, row: usize, col: usize) -> Option<&mut T> {
    //     if row >= self.row_count || col >= self.column_count {
    //         return None;
    //     }
    //     Some(unsafe { self.get_unchecked_mut(row, col) })
    // }

    /// Get a mutable slice for an entire row
    /// # Safety
    /// Caller must ensure that no other mutable references to elements in this row exist
    unsafe fn get_row_unchecked_mut(&mut self, row: usize) -> &mut [T] {
        &mut *self.rows[row]
        //&mut *(*row).as_mut_ptr()
    }

    /// Get a mutable slice for an entire row (bounds checked)
    pub fn get_row_mut(&mut self, row: usize) -> Option<&mut [T]> {
        if row >= self.row_count() {
            return None;
        }
        Some(unsafe { self.get_row_unchecked_mut(row) })
    }

    /// Get an immutable slice for an entire row
    pub fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.row_count() {
            return None;
        }
        Some(unsafe { &*self.rows[row] })
    }

    /// Fill all elements in the slice with the given value
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for row in 0..self.row_count() {
            if let Some(row_slice) = self.get_row_mut(row) {
                row_slice.fill(value.clone());
            }
        }
    }

    /// Apply a function to each element
    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for row in 0..self.row_count() {
            if let Some(row_slice) = self.get_row_mut(row) {
                for element in row_slice.iter_mut() {
                    f(element);
                }
            }
        }
    }

    /// Create an iterator over all elements (row by row)
    pub fn iter(&self) -> Vec2DMutSliceIter<T> {
        Vec2DMutSliceIter {
            slice: self,
            current_row: 0,
            current_col: 0,
        }
    }

    /// Copy data to a new Vec2D
    pub fn to_vec2d(&self) -> Vec2D<T>
    where
        T: Clone,
    {
        let mut items = Vec::with_capacity(self.len());
        for row in 0..self.row_count() {
            if let Some(row_slice) = self.get_row(row) {
                items.extend_from_slice(row_slice);
            }
        }
        Vec2D {
            size: self.size,
            items,
        }
    }
}

pub struct Vec2DMutSliceIter<'a, T> {
    slice: &'a Vec2DMutSlice<'a, T>,
    current_row: usize,
    current_col: usize,
}

impl<'a, T> Iterator for Vec2DMutSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.slice.row_count() {
            return None;
        }

        let result = self.slice.get(self.current_row, self.current_col);

        self.current_col += 1;
        if self.current_col >= self.slice.column_count() {
            self.current_col = 0;
            self.current_row += 1;
        }

        result
    }
}

pub struct Vec2DMutSliceIterMut<'a, T: 'a> {
    slice: Vec<&'a mut [T]>,
    current_row: usize,
    current_col: usize,
    column_count: usize
}

impl<'a, T> Iterator for Vec2DMutSliceIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.slice.len() {
            return None;
        }

        let result = Some(unsafe {&mut *((*self.slice[self.current_row]).as_mut_ptr().add(self.current_col))});

        self.current_col += 1;
        if self.current_col >= self.column_count {
            self.current_col = 0;
            self.current_row += 1;
        }

        result
    }
}

/// A 2D slice view that contains a vector of row slices
pub struct Vec2DSlice<'a, T> {
    row_slices: Vec<&'a [T]>,
    pub size: Size2D
}

impl<'a,T> HasSize2D for Vec2DSlice<'a, T> {

    #[inline]
    fn row_count(&self) -> usize {
        self.size.row_count
    }

    #[inline]
    fn column_count(&self) -> usize {
        self.size.column_count
    }

    #[inline]
    fn size(&self) -> Size2D {
        self.size
    }
}

impl<'a, T> Vec2DSlice<'a, T> {

    pub fn get(&self, row: usize, column: usize) -> Option<&'a T>
    {
        self.get_index2d(MemIndex2D::new(row, column))
    }

    pub fn get_index2d(&self, index2d: MemIndex2D) -> Option<&'a T> {
        if !self.index2d_in_bounds(&index2d) {
            return None;
        }

        self.row_slices.get(index2d.row)?.get(index2d.col)
    }

    /// Get the element at slice-relative coordinates, panicking if out of bounds
    pub fn get_unchecked(&self, row: usize, col: usize) -> &T {
        &self.row_slices[row][col]
    }

    /// Get a complete row slice within the 2D slice bounds
    pub fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.row_count() {
            return None;
        }

        Some(self.row_slices[row])
    }

    /// Iterator over all elements in the 2D slice (row by row)
    pub fn iter(&self) -> Vec2DSliceIter<T> {
        Vec2DSliceIter {
            slice: self,
            current_row: 0,
            current_col: 0,
        }
    }

    /// Iterator over all rows in the 2D slice
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        (0..self.row_count()).map(move |row| self.get_row(row).unwrap())
    }

    /// Collect all elements into a Vec (row by row order)
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.iter().cloned().collect()
    }

    /// Create a new Vec2D from this slice
    pub fn to_vec2d(&self) -> Vec2D<T>
    where
        T: Clone,
    {
        Vec2D {
            size: self.size,
            items: self.to_vec(),
        }
    }

    /// Apply a function to each element and collect results
    pub fn map<U, F>(&self, mut f: F) -> Vec<U>
    where
        F: FnMut(&T) -> U,
    {
        self.iter().map(|item| f(item)).collect()
    }

    /// Check if the slice contains a value
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|item| item == x)
    }
}

/// Iterator over elements in a Vec2DSlice
pub struct Vec2DSliceIter<'a, T> {
    slice: &'a Vec2DSlice<'a, T>,
    current_row: usize,
    current_col: usize,
}

impl<'a, T> Iterator for Vec2DSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.slice.row_count() {
            return None;
        }

        let result = self.slice.get(self.current_row, self.current_col);

        // Move to next position
        self.current_col += 1;
        if self.current_col >= self.slice.column_count() {
            self.current_col = 0;
            self.current_row += 1;
        }

        result
    }
}

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
        assert_eq!(slice.column_count(), 1);
        assert_eq!(slice.row_count(), 1);
        assert_eq!(slice.get(0, 0), Some(&4));

        assert_eq!([6,7,8].as_slice(), vec2d.get_row(2).unwrap());

        // Full row slice
        slice = vec2d.get_slice(MemSpan2D::new_from_usize(2, 0, 3, 3)).unwrap();
        assert_eq!(slice.column_count(), 3);
        assert_eq!(slice.row_count(), 1);
        assert_eq!(slice.get_row(0), Some([6, 7, 8].as_slice()));

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

        assert_eq!([1, 2].as_slice(), slice.get_row(0).unwrap());
        assert_eq!([4, 5].as_slice(), slice.get_row(1).unwrap());
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

