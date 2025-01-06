use std::ops::Range;

use super::{memory_extents2d::MemExtents2D, memory_index2d::MemIndex2D, memory_index_range::{DecreasingRange, IncreasingRange, MemoryIndexRange}, memory_iter::HasCurMemIndex, offset::Offset};

pub struct ClockwiseCornerIterator
{
    extents: MemExtents2D,
    corner: u8
}

impl Iterator for ClockwiseCornerIterator
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        let current: Option<MemIndex2D> = match self.corner
        {
            0 => Some(self.extents.get_min_coord()),
            1 => Some(MemIndex2D::new(self.extents.get_min_coord().row, self.extents.get_max_coord().col)),
            2 => Some(self.extents.get_max_coord()),
            3 => Some(MemIndex2D::new(self.extents.get_max_coord().row, self.extents.get_max_coord().col)),
            _ => None
        };

        self.corner += 1;
        current
    }
}

///TODO: Write unit tests for this shit
pub struct LeftToRightRead
{
    pub row_range: Range<usize>,
    pub col_range: IncreasingRange
}

impl LeftToRightRead
{
    pub fn max_coord(&self) -> MemIndex2D
    {
        MemIndex2D::new(self.row_range.end - 1, self.col_range.max - 1)
    }

    pub fn min_coord(&self) -> MemIndex2D
    {
        MemIndex2D::new(self.row_range.start, self.col_range.min)
    }

    pub fn validate_extents(extents: MemExtents2D) -> bool
    {
        let min_coord = extents.get_min_coord();
        let max_coord = extents.get_max_coord();

        return min_coord.row < max_coord.row && min_coord.col < max_coord.col;
    }
}

impl HasCurMemIndex for LeftToRightRead
{
    fn get_cur_mem_index(&self) -> MemIndex2D {
        let mut col_val: usize = self.col_range.cur;
        let mut row_val: usize = self.row_range.start;

        if col_val == self.col_range.max
        {
            col_val = self.col_range.min;
            row_val = self.row_range.start + 1;
        }

        MemIndex2D::new(row_val, col_val)
    }
}

impl TryFrom<MemExtents2D> for LeftToRightRead
{   
    type Error = &'static str;
    
    fn try_from(value: MemExtents2D) -> Result<Self, Self::Error> {
        let min_coord: MemIndex2D = value.get_min_coord();
        let max_coord: MemIndex2D = value.get_max_coord();

        if min_coord.row >= max_coord.row
        {
            return Err("min_row must be < max_row");
        }

        if min_coord.col >= max_coord.col
        {
            return Err("min_col must be < max_col");
        }

        Ok(LeftToRightRead
        {
            row_range: min_coord.row..max_coord.row,
            col_range: IncreasingRange::new(min_coord.col, max_coord.col)
        })
    }
}

impl Iterator for LeftToRightRead
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        match self.col_range.next()
        {
            Some(c_val) => Some(MemIndex2D::new(self.row_range.start, c_val)),
            None => 
            {
                match self.row_range.next()
                {
                    Some(_) => 
                    {
                        self.col_range.reset();

                        if self.row_range.start >= self.row_range.end
                        {
                            return None;
                        }

                        if let Some(c_val) = self.col_range.next()
                        {
                            return Some(MemIndex2D::new(self.row_range.start, c_val));
                        }
                        else 
                        {
                            return None;    
                        }
                        
                    },
                    None => {return None;}
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct RightToLeafRead
{
    pub row_range: IncreasingRange,
    pub col_range: DecreasingRange
}

impl Iterator for RightToLeafRead
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        match self.col_range.next()
        {
            Some(c_val) => Some(MemIndex2D::new(self.row_range.cur, c_val)),
            None => 
            {
                match self.row_range.next()
                {
                    Some(r_val) => 
                    {
                        //col range has to be valid here
                        self.col_range.reset();
                        return Some(MemIndex2D::new(r_val, self.col_range.cur));
                    },
                    None => {return None;}
                }
            }
        }
    }
}

impl RightToLeafRead
{
    pub fn new(min_coord: MemIndex2D, max_coord: MemIndex2D) -> Self
    {
        let row_range = IncreasingRange::new(min_coord.row, max_coord.row);

        let col_range = DecreasingRange::new(min_coord.col, max_coord.col);

        RightToLeafRead
        {
            row_range,
            col_range
        }
    }
}

///Reversed "as the ox plows" start at the bottom right corner
/// Then read right to left, move up 1 row and read left to right
/// continue untill finished 
pub struct ReverseBoustrophedonRead
{
    row_range: DecreasingRange,
    col_range: MemoryIndexRange
}

impl Iterator for ReverseBoustrophedonRead
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        match self.col_range.next()
        {
            Some(col_val) => Some(MemIndex2D::new(self.row_range.cur, col_val)),
            None => 
            {
                self.col_range.reset_flip();

                match self.row_range.next()
                {
                    Some(r_val) => Some(MemIndex2D::new(r_val, self.col_range.cur())),
                    None => None
                }
            }
        }
    }
}

impl ReverseBoustrophedonRead
{

    pub fn new_from_extents(extents: MemExtents2D, direction: Offset) -> Self
    {
        match direction
        {
            Offset::Pos(_) => Self::new_start_left_to_right(extents.get_min_coord(), extents.get_max_coord()),
            Offset::Neg(_) => Self::new_start_right_to_left(extents.get_min_coord(), extents.get_max_coord())
        }
    }

    pub fn new_start_left_to_right(min_index2d: MemIndex2D, max_index2d: MemIndex2D) -> Self
    {
        let row_range = DecreasingRange::new(max_index2d.row, min_index2d.row);

        let pos_range = IncreasingRange::new( min_index2d.col, max_index2d.col);

        let col_range = MemoryIndexRange::new_positive(pos_range);

        ReverseBoustrophedonRead
        {
            row_range,
            col_range
        }
    }
    pub fn new_start_right_to_left(min_index2d: MemIndex2D, max_index2d: MemIndex2D) -> Self
    {
        let row_range = DecreasingRange::new(max_index2d.row, min_index2d.row);

        let neg_range = DecreasingRange::new(max_index2d.col, min_index2d.col);

        let col_range = MemoryIndexRange::new_negative(neg_range);

        ReverseBoustrophedonRead
        {
            row_range,
            col_range
        }
    }
}

/// Means "as the ox plows" read left to right for the first row
/// then right to left for the next row, and so on untill completion
pub struct BoustrophedonRead
{
    row_range: IncreasingRange,
    col_range: MemoryIndexRange,
    current_direction: Offset
}

impl Iterator for BoustrophedonRead
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        match self.col_range.next()
        {
            Some(col_val) => Some(MemIndex2D::new(self.row_range.cur, col_val)),
            None => 
            {
                self.col_range.reset_flip();
                match self.row_range.next()
                {
                    Some(r_val) => Some(MemIndex2D::new(r_val, self.col_range.cur())),
                    None => None
                }
            }
        }
    }
}

impl BoustrophedonRead
{
    pub fn new(min_index2d: MemIndex2D, max_index2d: MemIndex2D) -> Self
    {
        let row_range = IncreasingRange::new(min_index2d.row, max_index2d.row);

        let pos_range = IncreasingRange::new(min_index2d.col, max_index2d.col);

        let col_range = MemoryIndexRange::new_positive(pos_range);

        BoustrophedonRead
        {
            row_range,
            col_range,
            current_direction: Offset::Pos(1)
        }
    }
}