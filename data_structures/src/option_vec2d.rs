use std::ops::{Index, IndexMut};

use memory_math::{
    memory_extents2d::MemExtents2D, memory_index2d::MemIndex2D, memory_range::LeftToRightRead,
};

use super::{vec2d::Vec2D, vec2d_iter::Vec2DIntoIter};

//OptionVec Contains a Vec<Option<T>>
pub struct OptionVec2D<T> {
    width: usize,
    height: usize,
    items: Vec<Option<T>>,
}

impl<T> IntoIterator for OptionVec2D<T> {
    type Item = Option<T>;
    type IntoIter = Vec2DIntoIter<Option<T>>;

    fn into_iter(self) -> Self::IntoIter {
        Vec2DIntoIter::new(self.items.into_iter(), self.width)
    }
}

impl<T> IndexMut<MemIndex2D> for OptionVec2D<T> {
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        match self.index2d_to_index(index) {
            Some(idx) => &mut self.items[idx],
            None => panic!("Index: {} out of bounds", index),
        }
    }
}

impl<T> Index<MemIndex2D> for OptionVec2D<T> {
    type Output = Option<T>;

    fn index(&self, index: MemIndex2D) -> &Self::Output {
        match self.index2d_to_index(index) {
            Some(idx) => &self.items[idx],
            None => panic!("Index: {} out of bounds!", index),
        }
    }
}

impl<T> OptionVec2D<T> {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    pub fn new(items: Vec<T>, width: usize, height: usize) -> Self {
        OptionVec2D {
            width,
            height,
            items: items
                .into_iter()
                .map(|e| -> Option<T> { Some(e) })
                .collect(),
        }
    }

    pub fn index2d_in_bounds(&self, index: MemIndex2D) -> bool {
        return index.row < self.height() && index.col < self.width();
    }

    pub fn index_to_index2d(&self, index: usize) -> Option<MemIndex2D> {
        if index >= self.len() {
            return None;
        }

        let row = index / self.width();
        let col = index % self.width();

        Some(MemIndex2D::new(row, col))
    }

    pub fn index2d_to_index(&self, coordinates: MemIndex2D) -> Option<usize> {
        if coordinates.row >= self.height() || coordinates.col >= self.width() {
            return None;
        }

        Some(coordinates.row * self.width() + coordinates.col)
    }
}

impl<T: Clone> Clone for OptionVec2D<T> {
    fn clone(&self) -> Self {
        Self {
            width: self.width.clone(),
            height: self.height.clone(),
            items: self.items.clone(),
        }
    }
}

impl<T> OptionVec2D<T> {
    pub fn take_at(&mut self, index2d: MemIndex2D) -> Result<T, String> {
        let index: usize = match self.index2d_to_index(index2d) {
            Some(ind) => ind,
            None => {
                return Err(format!(
                    "Cannot take value at: {} index out of bounds!",
                    index2d
                ));
            }
        };

        match std::mem::replace(&mut self.items[index], None) {
            Some(cell) => Ok(cell),
            None => {
                return Err(format!("Failed to take cell at {}, value was None", index));
            }
        }
    }

    pub fn take_range(&mut self, range: MemExtents2D) -> Result<Vec2D<T>, String> {
        let iter = match LeftToRightRead::try_from(range) {
            Ok(itr) => itr,
            Err(msg) => {
                return Err(format!("Failed to take range, invalid range: {}", msg));
            }
        };

        let mut items: Vec<T> = Vec::with_capacity(range.area());
        for index in iter {
            match self.take_at(index) {
                Ok(val) => items.push(val),
                Err(msg) => return Err(format!("Failed to take range: {}", msg)),
            }
        }

        Vec2D::new_from_flatpack(items, range.width(), range.height())
    }
}

impl<T: Clone> OptionVec2D<T> {
    pub fn new_empty(extents: MemExtents2D) -> OptionVec2D<T> {
        let items: Vec<Option<T>> = vec![None; extents.area()];
        OptionVec2D {
            width: extents.width(),
            height: extents.height(),
            items,
        }
    }

    pub fn clone_cell_range(&self, range: MemExtents2D) -> Result<OptionVec2D<T>, String> {
        let iterator: LeftToRightRead = match LeftToRightRead::try_from(range) {
            Ok(itr) => itr,
            Err(msg) => return Err(format!("Cannot clone invalid range: {}", msg)),
        };

        let max = iterator.max_coord();
        let min = iterator.min_coord();

        if !self.index2d_in_bounds(max) || !self.index2d_in_bounds(min) {
            return Err(format!(
                "Attempted to get range: {} which is out of bounds",
                range
            ));
        }

        let mut items: OptionVec2D<T> = Self::new_empty(range);

        for coordinate in iterator {
            let index = match self.index2d_to_index(coordinate) {
                Some(ind) => ind,
                None => {
                    return Err(format!(
                        "Cannot clone cell at: {} out of bounds",
                        coordinate
                    ));
                }
            };

            let cell = match &self.items[index] {
                Some(cell_val) => Some(cell_val.clone()),
                None => None,
            };

            items[coordinate] = cell;
        }

        Ok(items)
    }
}
