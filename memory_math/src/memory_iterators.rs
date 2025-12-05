use std::ops::Range;
use super::{memory_span2d::{MemSpan2D, HasMemSpan2D}, memory_index2d::MemIndex2D, memory_range_iter::HasCurMemIndex};

pub trait MemoryIterator : Iterator<Item=MemIndex2D>
{
    fn current_index(&self) -> Option<MemIndex2D>;
}

pub struct ClockwiseCornerIterator
{
    extents: MemSpan2D,
    corner: u8
}

impl Iterator for ClockwiseCornerIterator
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current_index();
        self.corner += 1;
        current
    }
}

impl MemoryIterator for ClockwiseCornerIterator
{
    fn current_index(&self) -> Option<MemIndex2D>
    {
        match self.corner
        {
            0 => Some(self.extents.min_absolute_index2d()),
            1 => Some(MemIndex2D::new(self.extents.min_row(), self.extents.max_column()?)),
            2 => Some(self.extents.max_absolute_index2d()?),
            3 => Some(MemIndex2D::new(self.extents.max_row()?, self.extents.max_column()?)),
            _ => None
        }
    }
}

pub struct LinearMemoryIterator
{
    extents: MemSpan2D,
    current_index: MemIndex2D
}

impl MemoryIterator for LinearMemoryIterator
{
    fn current_index(&self) -> Option<MemIndex2D>
    {
        if self.current_index.row > self.extents.max_row()?
        {
            return None;
        }

        Some(self.current_index)
    }
}

impl Iterator for LinearMemoryIterator
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        let next_col: usize = self.next_column();
        let next_row: usize = if next_col == 0 { self.next_row()? } else { self.current_index.row };

        self.current_index = MemIndex2D::new(next_row, next_col);
        Some(self.current_index)
    }
}

impl DoubleEndedIterator for LinearMemoryIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_col = self.prev_column()?;
        let next_row: usize;

        if next_col == self.extents.max_column()? {
            next_row = self.current_index.row.checked_sub(1)?;
            if next_row < self.extents.min_row() {
                return None;
            }
        }
        else {
            next_row = self.current_index.row;
        }

        self.current_index = MemIndex2D::new(next_row, next_col);
        Some(self.current_index)
    }
}

impl HasCurMemIndex for LinearMemoryIterator
{
    fn get_cur_mem_index(&self) -> MemIndex2D {
        self.current_index
    }
}

impl From<MemSpan2D> for LinearMemoryIterator
{
    fn from(extents: MemSpan2D) -> Self {
        LinearMemoryIterator::new(extents)
    }
}

impl LinearMemoryIterator
{
    pub fn new(extents: MemSpan2D) -> Self
    {
        LinearMemoryIterator
        {
            extents,
            current_index: MemIndex2D::new(0, 0)
        }
    }

    ///TODO: Modify so it exhibits the variable++ behaviour
    ///check THEN assign
    fn next_column(&mut self) -> usize
    {
        let col: usize = self.current_index.col;
        self.current_index.col += 1;
        if col > self.extents.max_column().unwrap()
        {
            self.current_index.col = 0;
            return 0;
        }

        col
    }

    fn prev_column(&mut self) -> Option<usize>
    {
        let col: usize = self.current_index.col;
        let mut set: bool = false;

        //slightly more complicated here, treat (None) as < min column
        if let Some(c) = self.current_index.col.checked_sub(1)
        {
            if c >= self.extents.min_column()
            {
                set = true;
                self.current_index.col = c;
            }
        }

        if !set
        {
            self.current_index.col = self.extents.max_column()?;
        }

        Some(col)
    }

    fn next_row(&mut self) -> Option<usize>
    {
        let row: usize = self.current_index.row;
        self.current_index.row += 1;
        if row > self.extents.max_row()?
        {
            return None;
        }

        Some(row)
    }
}

///"as the ox plows" start (0,0) then read to the end of the row
/// then move to next row, and read the columns backwards and so on.
/// TODO: We could make this more efficient by removing the number of
/// unwrap() checks we have to do.
pub struct BoustrophedonIterator
{
    range2d: MemSpan2D,
    current_index: MemIndex2D
}

impl Iterator for BoustrophedonIterator
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.next_column()
        {
            return Some(MemIndex2D::new(self.current_index.row, c));
        }
        let row: usize = self.next_row()?;
        let col: usize = self.range2d.max_column()?;
        Some(MemIndex2D::new(row, col))
    }
}

impl DoubleEndedIterator for BoustrophedonIterator
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.prev_column()
        {
            return Some(MemIndex2D::new(self.current_index.row, c));
        }
        let row: usize = self.next_row()?;
        let col: usize = self.range2d.min_column();
        Some(MemIndex2D::new(row, col))
    }
}

impl BoustrophedonIterator
{

    fn next_row(&self) -> Option<usize>
    {
        let row: usize = self.current_index.row.checked_add(1)?;

        if row >= self.range2d.max_row()?
        {
            return None;
        }

        Some(row)
    }


    fn prev_row(&self) -> Option<usize>
    {
        let row: usize = self.current_index.row.checked_sub(1)?;

        if row < self.range2d.min_row()
        {
            return None;
        }

        Some(row)
    }

    fn prev_column(&self) -> Option<usize>
    {
        let col: usize;
        if self.current_index.row % 2 == 0
        {
            col = self.current_index.col.checked_sub(1)?;
        }
        else {
            col = self.current_index.col.checked_add(1)?;
        }

        if col >= self.range2d.max_column()?
        {
            return None;
        }
        else if col < self.range2d.min_column()
        {
            return None;
        }

        Some(col)
    }


    fn next_column(&self) -> Option<usize>
    {
        let col: usize;
        if self.current_index.row % 2 == 0
        {
            col = self.current_index.col.checked_add(1)?;
        }
        else {
            col = self.current_index.col.checked_sub(1)?;
        }

        if col > self.range2d.max_column()?
        {
            return None;
        }
        else if col < self.range2d.min_column()
        {
            return None;
        }


        Some(col)
    }
}