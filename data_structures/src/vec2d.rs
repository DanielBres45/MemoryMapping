use std::ops::{Index, IndexMut};

use memory_math::{
    memory_extents2d::{HasMemExtents2D, MemExtents2D},
    memory_index2d::MemIndex2D,
    memory_iter::IterateWithMemIndex,
    memory_range::LeftToRightRead,
    memory_vect2d::MemVect2D,
};

use super::{iter_index2d::CanIterIndex2D, vec2d_iter::Vec2DIntoIter, vec_2d::Vector2D};

#[derive(Clone)]
pub struct Vec2D<T> {
    width: usize,
    height: usize,
    items: Vec<T>,
}

impl<T> HasMemExtents2D for Vec2D<T> {
    fn get_extents(&self) -> Result<MemExtents2D, &'static str> {
        Ok(self.extents())
    }
}

impl<T> CanIterIndex2D for Vec2D<T> {}

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
        match self.get_ref(index) {
            Some(val) => val,
            None => panic!(
                "Vev2d Coordinates out of bounds. Coordinate was {} but the size is {}",
                index,
                self.extents()
            ),
        }
    }
}

impl<T> IndexMut<MemIndex2D> for Vec2D<T> {
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        let extents = self.extents();
        match self.get_mut(index) {
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

impl<T: Clone> Vec2D<T> {
    pub fn new_from_extents_reference(extents: MemExtents2D, ref_item: &T) -> Self {
        Self::new_size_reference(extents.width(), extents.height(), ref_item)
    }

    pub fn new_size_reference(width: usize, height: usize, ref_item: &T) -> Self {
        let items = vec![ref_item.clone(); width * height];
        Vec2D {
            width,
            height,
            items,
        }
    }
}

impl<T> Vector2D<T> for Vec2D<T> {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }

    fn new(items: Vec<T>, width: usize, height: usize) -> Self {
        Vec2D {
            items,
            width,
            height,
        }
    }
}

impl<T> Vec2D<T> {
    pub fn get_mut(&mut self, coordinates: MemIndex2D) -> Option<&mut T> {
        let index = match self.index2d_to_index(coordinates) {
            Some(val) => val,
            None => return None,
        };

        Some(self.items.get_mut(index).unwrap())
    }

    pub fn get_ref(&self, coordinates: MemIndex2D) -> Option<&T> {
        let index = match self.index2d_to_index(coordinates) {
            Some(val) => val,
            None => return None,
        };

        Some(&self.items[index])
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
}

