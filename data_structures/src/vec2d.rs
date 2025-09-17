use std::cmp::max;
use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

extern crate proc_macro;
use proc_macro::TokenStream;

use memory_math::{
    memory_extents2d::{HasMemExtents2D, MemExtents2D},
    memory_index2d::MemIndex2D,
    memory_iter::IterateWithMemIndex,
    memory_range::LeftToRightRead,
    memory_vect2d::MemVect2D,
};

use super::{vec2d_iter::Vec2DIntoIter};

pub trait Vec2DMethods<T> : HasMemExtents2D
{
    fn len(&self) -> usize;
    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T>;
    #[inline]
    fn index2d_in_bounds(&self, index: MemIndex2D) -> bool {
        return index.row < self.height() && index.col < self.width();
    }
    fn index_to_index2d(&self, index: usize) -> Option<MemIndex2D> {
        if index >= self.len() {
            return None;
        }

        let row = index / self.width();
        let col = index % self.width();

        Some(MemIndex2D::new(row, col))
    }

    fn index2d_to_index(&self, index2d: MemIndex2D) -> Option<usize> {
        if index2d.row >= self.height() || index2d.col >= self.width() {
            return None;
        }

        Some(index2d.row * self.width() + index2d.col)
    }

    /// Get a reference to an element at 2D coordinates
    fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.get_index2d(MemIndex2D::new(row, col))
    }
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
    width: usize,
    height: usize,
    items: Vec<T>,
}

impl<T> HasMemExtents2D for Vec2D<T> {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}
impl<T> Vec2DMethods<T> for Vec2D<T> {
    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }
    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T> {
        self.index2d_to_index(coordinates).and_then(|i| self.items.get(i))
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
        Vec2DIntoIter::new(self.items.into_iter(), self.width)
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
                self.extents()
            ),
        }
    }
}

impl<T> IndexMut<MemIndex2D> for Vec2D<T> {
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        let extents = self.extents();
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
    pub fn new_from_flatpack(flatpack: Vec<T>, width: usize, height: usize) -> Result<Self, String>
    where
        Self: Sized,
    {
        if flatpack.len() % width * height != 0 {
            return Err("flatpack vec has improper size for width, and height".to_string());
        }

        Ok(Self::new(flatpack, width, height))
    }

    pub fn new_size_reference(width: usize, height: usize, ref_item: &T) -> Self
    where
        T: Clone,
        Self: Sized,
    {
        let items = vec![ref_item.clone(); width * height];

        Self::new(items, width, height)
    }

    pub fn new_from_extents_reference(extents: MemExtents2D, ref_item: &T) -> Self
    where
        T: Clone,
        Self: Sized,
    {
        Self::new_size_reference(extents.width(), extents.height(), ref_item)
    }

    pub fn from_vec(items: Vec<T>, width: usize) -> Option<Self> {
        if items.len() % width != 0 {
            return None;
        }

        let height = items.len() / width;
        Some(Self {
            width,
            height,
            items,
        })
    }

    pub fn new(items: Vec<T>, width: usize, height: usize) -> Self {
        Vec2D {
            items,
            width,
            height,
        }
    }


    pub fn push_range(&mut self, start_index: MemIndex2D, range: Vec2D<T>) {
        if start_index.col > self.width || start_index.row > self.height {
            return;
        }

        let shift: MemVect2D = MemVect2D::from(start_index);

        for (index, item) in range.into_iter().iterate_with_mem_index() {
            let self_index: MemIndex2D = index + shift;
            self[self_index] = item;
        }
    }

    /// Get a slice of a complete row
    pub fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.height {
            return None;
        }

        let start = row * self.width;
        let end = start + self.width;
        Some(&self.items[start..end])
    }

    pub fn get_row_slice(&self, row: usize, min_col: usize, max_col: usize) -> Option<&[T]> {
        if row >= self.height || max_col >= self.width || max_col < min_col {
            return None;
        }

        let start = row * self.width + min_col;
        let end = row * self.width + max_col + 1;

        Some(&self.items[start..end])
    }

    /// Get a mutable slice of a complete row
    pub fn get_row_mut(&mut self, row: usize) -> Option<&mut [T]> {
        if row >= self.height {
            return None;
        }

        let start = row * self.width;
        let end = start + self.width;
        Some(&mut self.items[start..end])
    }

    pub fn get_row_slice_mut(&mut self, row: usize, min_col: usize, max_col: usize) -> Option<&mut [T]> {
        if row >= self.height || max_col >= self.width || max_col < min_col {
            return None;
        }

        let start = row * self.width + min_col;
        let end = row * self.width + max_col + 1;
        Some(&mut self.items[start..end])
    }

    /// Get a 1D slice view of a rectangular region
    pub fn get_slice(&self, min_row: usize, min_col: usize, max_row: usize, max_col: usize) -> Option<Vec2DSlice<T>> {
        if min_row > max_row || min_col > max_col ||
            max_row >= self.height || max_col >= self.width {
            return None;
        }

        let mut row_slices = Vec::new();
        for row in min_row..=max_row {
            if let Some(row_slice) = self.get_row_slice(row, min_col, max_col) {
                row_slices.push(row_slice);
            } else {
                return None;
            }
        }

        Some(Vec2DSlice {
            row_slices,
            width: max_col - min_col + 1,
            height: max_row - min_row + 1
        })
    }

    pub fn get_slice_mut(&mut self, min_row: usize, min_col: usize, max_row: usize, max_col: usize) -> Option<Vec2DMutSlice<T>> {
        if min_row > max_row || min_col > max_col ||
            max_row >= self.height || max_col >= self.width {
            return None;
        }

        let mut row_slices: Vec<*mut [T]> = Vec::new();
        for row in min_row..=max_row {
            if let Some(row_slice) = self.get_row_slice_mut(row, min_col, max_col) {
                row_slices.push(row_slice as *mut [T]);
            } else {
                return None;
            }
        }

        Some(Vec2DMutSlice {
            rows: row_slices,
            width: max_col - min_col + 1,
            height: max_row - min_row + 1,
            _marker: std::marker::PhantomData,
        })
    }
}

pub struct Vec2DMutSlice<'a, T: 'a> {
    /// # Safety
    /// This slice pointer must point at a valid region of `T` with at least length `v.len()`. Normally,
    /// those requirements would mean that we could instead use a `&mut [T]` here, but we cannot
    /// because `__iterator_get_unchecked` needs to return `&mut [T]`, which guarantees certain aliasing
    /// properties that we cannot uphold if we hold on to the full original `&mut [T]`. Wrapping a raw
    /// slice instead lets us hand out non-overlapping `&mut [T]` subslices of the slice we wrap.
    rows: Vec<*mut [T]>,
    width: usize,
    height: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> HasMemExtents2D for Vec2DMutSlice<'a, T>
{
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<'a, T> Vec2DMethods<T> for Vec2DMutSlice<'a, T> {
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn get_index2d(&self, coordinates: MemIndex2D) -> Option<&T> {
        self.get(coordinates.row, coordinates.col)
    }
}

impl<'a, T> MutVec2DMethods<T> for Vec2DMutSlice<'a, T> {
    fn get_mut_index2d(&mut self, coordinates: MemIndex2D) -> Option<&mut T> {
        self.get_mut(coordinates.row, coordinates.col)
    }
}

impl<'a, T> Vec2DMutSlice<'a, T> {
    /// Get the width of this 2D slice
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of this 2D slice
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a reference to an element at slice-relative coordinates
    /// # Safety
    /// Caller must ensure that row and col are within bounds
    pub unsafe fn get_unchecked(&self, row: usize, col: usize) -> &T {
        let row_ptr = self.rows[row];
        &*(*row_ptr).as_ptr().add(col)
    }

    /// Get a mutable reference to an element at slice-relative coordinates
    /// # Safety
    /// Caller must ensure that row and col are within bounds and that no other
    /// mutable references to the same element exist
    pub unsafe fn get_unchecked_mut(&self, row: usize, col: usize) -> &mut T {
        let row_ptr = self.rows[row];
        &mut *(*row_ptr).as_mut_ptr().add(col)
    }

    /// Get a reference to an element at slice-relative coordinates (bounds checked)
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.height || col >= self.width {
            return None;
        }
        Some(unsafe { self.get_unchecked(row, col) })
    }

    /// Get a mutable reference to an element at slice-relative coordinates (bounds checked)
    pub fn get_mut(&self, row: usize, col: usize) -> Option<&mut T> {
        if row >= self.height || col >= self.width {
            return None;
        }
        Some(unsafe { self.get_unchecked_mut(row, col) })
    }

    /// Get a mutable slice for an entire row
    /// # Safety
    /// Caller must ensure that no other mutable references to elements in this row exist
    unsafe fn get_row_unchecked_mut(&mut self, row: usize) -> &mut [T] {
        &mut *self.rows[row]
        //&mut *(*row).as_mut_ptr()
    }

    /// Get a mutable slice for an entire row (bounds checked)
    pub fn get_row_mut(&mut self, row: usize) -> Option<&mut [T]> {
        if row >= self.height {
            return None;
        }
        Some(unsafe { self.get_row_unchecked_mut(row) })
    }

    /// Get an immutable slice for an entire row
    pub fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.height {
            return None;
        }
        Some(unsafe { &*self.rows[row] })
    }

    /// Fill all elements in the slice with the given value
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for row in 0..self.height {
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
        for row in 0..self.height {
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

    /// Create a mutable iterator over all elements (row by row)
    pub fn iter_mut(&self) -> Vec2DMutSliceIterMut<T> {
        Vec2DMutSliceIterMut {
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
        let mut items = Vec::with_capacity(self.width * self.height);
        for row in 0..self.height {
            if let Some(row_slice) = self.get_row(row) {
                items.extend_from_slice(row_slice);
            }
        }
        Vec2D {
            width: self.width,
            height: self.height,
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
        if self.current_row >= self.slice.height {
            return None;
        }

        let result = self.slice.get(self.current_row, self.current_col);

        self.current_col += 1;
        if self.current_col >= self.slice.width {
            self.current_col = 0;
            self.current_row += 1;
        }

        result
    }
}

pub struct Vec2DMutSliceIterMut<'a, T> {
    slice: &'a Vec2DMutSlice<'a, T>,
    current_row: usize,
    current_col: usize,
}

impl<'a, T> Iterator for Vec2DMutSliceIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.slice.height {
            return None;
        }

        let result = unsafe {
            // Safety: We ensure we never return the same mutable reference twice
            // by advancing our position after each call
            self.slice.get_unchecked_mut(self.current_row, self.current_col)
        };

        self.current_col += 1;
        if self.current_col >= self.slice.width {
            self.current_col = 0;
            self.current_row += 1;
        }

        Some(result)
    }
}

/// A 2D slice view that contains a vector of row slices
pub struct Vec2DSlice<'a, T> {
    row_slices: Vec<&'a [T]>,
    width: usize,
    height: usize,
}

impl<'a, T> Vec2DSlice<'a, T> {
    /// Get the width of this 2D slice
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of this 2D slice
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a reference to an element at slice-relative coordinates
    /// (0, 0) refers to the top-left corner of the slice
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.height() || col >= self.width() {
            return None;
        }

        self.row_slices.get(row)?.get(col)
    }

    /// Get the element at slice-relative coordinates, panicking if out of bounds
    pub fn get_unchecked(&self, row: usize, col: usize) -> &T {
        &self.row_slices[row][col]
    }

    /// Get a complete row slice within the 2D slice bounds
    pub fn get_row(&self, row: usize) -> Option<&[T]> {
        if row >= self.height() {
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
        (0..self.height()).map(move |row| self.get_row(row).unwrap())
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
            width: self.width(),
            height: self.height(),
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
        if self.current_row >= self.slice.height() {
            return None;
        }

        let result = self.slice.get(self.current_row, self.current_col);

        // Move to next position
        self.current_col += 1;
        if self.current_col >= self.slice.width() {
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

        let row = vec2d.get_row_slice(1, 1, 2);
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

        let row = vec2d.get_row_slice_mut(1, 1, 2).unwrap();
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
        let slice = vec2d.get_slice(1, 1, 2, 2).unwrap();

        assert_eq!(slice.width(), 2);
        assert_eq!(slice.height(), 2);

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
        let slice = vec2d.get_slice_mut(1, 1, 2, 2).unwrap();

        assert_eq!(slice.width(), 2);
        assert_eq!(slice.height(), 2);

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
        let vec2d = Vec2D::new(items, 3, 4); // 3x4 grid

        //grid looks like:
        // [0, 1, 2]
        // [3, 4, 5]
        // [6, 7, 8]
        // [9,10,11]

        // Single element slice
        let mut slice = vec2d.get_slice(1, 1, 1, 1).unwrap();
        assert_eq!(slice.width(), 1);
        assert_eq!(slice.height(), 1);
        assert_eq!(slice.get(0, 0), Some(&4));

        assert_eq!([6,7,8].as_slice(), vec2d.get_row(2).unwrap());

        // Full row slice
        slice = vec2d.get_slice(2, 0, 2, 2).unwrap();
        assert_eq!(slice.width(), 3);
        assert_eq!(slice.height(), 1);
        assert_eq!(slice.get_row(0), Some([6, 7, 8].as_slice()));

        // Invalid bounds should return None
        assert!(vec2d.get_slice(0, 0, 5, 5).is_none());
        assert!(vec2d.get_slice(2, 1, 1, 1).is_none()); // min > max
    }

    #[test]
    fn test_slice_edge_cases_mut() {
        let items: Vec<i32> = (0..12).collect();
        let mut vec2d = Vec2D::new(items, 3, 4); // 3x4 grid

        //grid looks like:
        // [0, 1, 2]
        // [3, 4, 5]
        // [6, 7, 8]
        // [9,10,11]

        // Single element slice
        let mut slice = vec2d.get_slice_mut(1, 1, 1, 1).unwrap();
        assert_eq!(slice.width(), 1);
        assert_eq!(slice.height(), 1);
        assert_eq!(slice.get(0, 0), Some(&4));

        assert_eq!([6, 7, 8].as_slice(), vec2d.get_row(2).unwrap());

        // Full row slice
        slice = vec2d.get_slice_mut(2, 0, 2, 2).unwrap();
        assert_eq!(slice.width(), 3);
        assert_eq!(slice.height(), 1);
        assert_eq!(slice.get_row(0), Some([6, 7, 8].as_slice()));

        // Invalid bounds should return None
        assert!(vec2d.get_slice(0, 0, 5, 5).is_none());
        assert!(vec2d.get_slice(2, 1, 1, 1).is_none()); // min > max
    }


    #[test]
    fn test_slice_methods() {
        let items: Vec<i32> = (1..=9).collect();
        let vec2d = Vec2D::from_vec(items, 3).unwrap(); // 3x3 grid with values 1-9

        let slice = vec2d.get_slice(0, 0, 1, 1).unwrap(); // 2x2 top-left corner

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
}

