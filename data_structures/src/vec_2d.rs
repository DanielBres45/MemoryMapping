use std::process::ExitCode;

use crate::memory_math::{memory_extents2d::MemExtents2D, memory_index2d::MemIndex2D};


pub trait Vector2D<T: Sized>
{
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn extents(&self) -> MemExtents2D
    {
        MemExtents2D::new_width_height(self.width(), self.height())
    }

    fn new(items: Vec<T>, width: usize, height: usize) -> Self;

    fn len(&self) -> usize;

    fn index2d_in_bounds(&self, index: MemIndex2D) -> bool
    {
        return index.row < self.height() && index.col < self.width()
    }

    fn index_to_index2d(&self, index: usize) -> Option<MemIndex2D>
    {
        if index >= self.len()
        {
            return None;
        }
    
        let row = index / self.width();
        let col = index % self.width();

        Some(MemIndex2D::new(row, col))
    }

    fn index2d_to_index(&self, coordinates: MemIndex2D) -> Option<usize>
    {
        if coordinates.row >= self.height() || coordinates.col >= self.width()
        {
            return None;
        }

        Some(coordinates.row * self.width() + coordinates.col)
    }

    fn new_from_flatpack(flatpack: Vec<T>, width: usize, height: usize) -> Result<Self, String> where Self: Sized
    {
        if flatpack.len() % width * height != 0
        {
            return Err("flatpack vec has improper size for width, and height".to_string());
        }

        Ok(Self::new(flatpack, width, height))
    }

    fn new_size_reference(width: usize, height: usize, ref_item: &T) -> Self
    where T: Clone,
    Self: Sized
    {
        let items = vec![ref_item.clone(); width * height];

        Self::new(items, width, height)
    }

    fn new_from_extents_reference(extents: MemExtents2D, ref_item: &T) -> Self
    where T:Clone,
    Self: Sized
    {
        Self::new_size_reference(extents.width(), extents.height(), ref_item)
    }

}