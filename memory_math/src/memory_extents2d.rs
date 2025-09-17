

use std::{fmt, ops::{Add, Sub}};

use super::{memory_index2d::MemIndex2D, memory_vect2d::MemVect2D};

#[derive(Copy, Clone)]
pub struct MemExtents2D
{
    min_coord: MemIndex2D,
    max_coord: MemIndex2D
}

impl PartialEq for MemExtents2D
{
    fn eq(&self, other: &Self) -> bool {
        self.min_coord == other.min_coord && self.max_coord == other.max_coord
    }
}

impl fmt::Debug for MemExtents2D
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemExtents2D").field("min_coord", &self.min_coord).field("max_coord", &self.max_coord).finish()
    }
}

pub trait HasMemExtents2D
{
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    #[inline]
    fn extents(&self) -> MemExtents2D {
        MemExtents2D::new_width_height(self.width(), self.height())
    }
}

impl HasMemExtents2D for MemExtents2D
{
    fn width(&self) -> usize {
        self.max_coord.col - self.min_coord.col
    }

    fn height(&self) -> usize {
        self.max_coord.row - self.min_coord.row
    }
}

pub trait HasComponentMemExtents2D
{
    fn get_component_extents(&self) -> Vec<MemExtents2D>;
}

impl fmt::Display for MemExtents2D
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "({},{})", self.min_coord, self.max_coord)
    }
}

impl Add<MemVect2D> for MemExtents2D
{
    //If the MemExtents where already invalid, this will fail. 
    type Output = MemExtents2D;

    fn add(self, rhs: MemVect2D) -> Self::Output {
        MemExtents2D
        {
            min_coord: self.min_coord + rhs,
            max_coord: self.max_coord + rhs
        }
    }
}

impl Sub<MemVect2D> for MemExtents2D
{
    type Output = Option<MemExtents2D>;

    fn sub(self, rhs: MemVect2D) -> Self::Output {
        match (self.min_coord - rhs, self.max_coord - rhs)
        {
            (Some(min), Some(max)) => 
            Some(MemExtents2D
            {
                min_coord: min,
                max_coord: max
            }),
            _ => None
        }
    }
}

impl MemExtents2D{

    pub fn range_coordinate_to_index(&self, coordinate: MemIndex2D) -> Option<usize>
    {
        if !self.contains_exclusive(coordinate)
        {
            return None;
        }

        Some((coordinate.row - self.min_coord.row) * self.width() + (coordinate.col - self.min_coord.col))
    }

    pub fn get_min_coord(&self) -> MemIndex2D
    {
        self.min_coord
    }

    pub fn get_max_coord(&self) -> MemIndex2D
    {
        self.max_coord
    }

    #[inline]
    pub fn validate_coordinates(min_coord: MemIndex2D, max_coord: MemIndex2D) -> bool
    {
        MemVect2D::from(min_coord) <= MemVect2D::from(max_coord)
    }

    pub fn new_width_height(width: usize, height: usize) -> Self
    {
        MemExtents2D
        {
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(height, width)
        }
    }

    pub fn new_from_coords(min_coord: MemIndex2D, max_coord: MemIndex2D) -> Result<Self, &'static str>
    {
        if !Self::validate_coordinates(min_coord, max_coord)
        {
            return Err("min_coord must be less than max_coord");
        }

        return Ok(MemExtents2D{
            min_coord,
            max_coord
        })
    }

    pub fn height(&self) -> usize
    {
        self.max_coord.row - self.min_coord.row
    }

    pub fn width(&self) -> usize
    {
        self.max_coord.col - self.min_coord.col
    }

    pub fn area(&self) -> usize
    {
        self.width() * self.height()
    }

    pub fn new_from_usize(min_row: usize, min_col: usize, max_row: usize, max_col: usize) -> Result<Self, &'static str>
    {
        return Self::new_from_coords(
            MemIndex2D::new(min_row, min_col), 
            MemIndex2D::new(max_row, max_col));
    }

    pub fn new_default() -> Self
    {
        MemExtents2D
        {
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(1, 1)
        }
    }

    fn update_min(&mut self, min_query: MemIndex2D)
    {
        self.min_coord.row = usize::min(self.min_coord.row, min_query.row);
        self.min_coord.col = usize::min(self.min_coord.col, min_query.col);
    }

    pub fn update_max(&mut self, max_query: MemIndex2D)
    {
        self.max_coord.row = usize::max(self.max_coord.row, max_query.row);
        self.max_coord.col = usize::max(self.max_coord.col, max_query.col);
    }

    ///Attempt to expand the extents to fit the new_coord 
    pub fn try_expand(&mut self, new_coord: MemIndex2D)
    {
        self.update_min(new_coord);
        self.update_max(new_coord);
    }

    pub fn try_shrink(&mut self, new_coord: MemIndex2D)
    {
        if !self.contains_inclusive(new_coord)
        {
            return;
        }

        self.update_min(new_coord);
        self.update_max(new_coord);
    }

    pub fn contains_exclusive(&self, index: MemIndex2D) -> bool{
        self.min_coord <= index && index < self.max_coord
    }

    pub fn contains_inclusive(&self, index: MemIndex2D) -> bool{
        return self.min_coord <= index && index <= self.max_coord
    }

    pub fn intersects(&self, other: MemExtents2D) -> bool
    {
        match self.intersect(other)
        {
            Some(_) => true,
            None => false
        }
    }

    pub fn intersect(&self, other: MemExtents2D) -> Option<Self>
    {
        // disjoint columnwise
        if self.min_coord.col > other.max_coord.col || other.min_coord.col > self.max_coord.col
        {
            return None;
        }

        //disjoint rowwise
        if self.min_coord.row > other.max_coord.row || other.min_coord.row > self.max_coord.row
        {
            return None;
        }

        let min_col: usize = usize::max(self.min_coord.col, other.min_coord.col);
        let min_row: usize = usize::max(self.min_coord.row, other.min_coord.row);

        let max_col: usize = usize::min(self.max_coord.col, other.max_coord.col);
        let max_row: usize = usize::min(self.max_coord.row, other.max_coord.row);

        Some(MemExtents2D
        {
            min_coord: MemIndex2D::new(min_row, min_col),
            max_coord: MemIndex2D::new(max_row, max_col)
        })
    }

    pub fn coordinate2d_to_index2d_in_extents(&self, index2d: MemIndex2D) -> Option<MemIndex2D>
    {
        if !self.contains_inclusive(index2d)
        {
            return None;
        }

        index2d - MemVect2D::from(self.min_coord)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_intersect_col_disjoint()
    {
        let left = MemExtents2D{
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(10, 10)
        };

        let right = MemExtents2D
        {
            min_coord: MemIndex2D::new(0, 11),
            max_coord: MemIndex2D::new(10, 21)
        };

        assert!(!left.intersects(right));
    }

    #[test]
    fn test_intersect_col_overlap()
    {
        let left = MemExtents2D{
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(10, 10)
        };

        let right = MemExtents2D
        {
            min_coord: MemIndex2D::new(0, 10),
            max_coord: MemIndex2D::new(10, 20)
        };

        assert!(left.intersects(right));
    }

    #[test]
    fn test_intersect_row_disjoint()
    {
        let bottom = MemExtents2D
        {
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(10, 10)
        };

        let top = MemExtents2D
        {
            min_coord: MemIndex2D::new(11, 0),
            max_coord: MemIndex2D::new(21, 10)
        };

        assert!(!bottom.intersects(top));
    }

    #[test]
    fn test_intersect_row_overlap()
    {
        let bottom = MemExtents2D
        {
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(10, 10)
        };

        let top = MemExtents2D
        {
            min_coord: MemIndex2D::new(10, 0),
            max_coord: MemIndex2D::new(20, 10)
        };

        assert!(bottom.intersects(top));
    }

    #[test]
    fn test_intersect_inside()
    {
        let outer = MemExtents2D
        {
            min_coord: MemIndex2D::origin(),
            max_coord: MemIndex2D::new(10, 10)
        };

        let inner = MemExtents2D
        {
            min_coord: MemIndex2D::new(5, 5),
            max_coord: MemIndex2D::new(7, 7)
        };

        assert!(outer.intersects(inner));
    }
}