

use std::mem;

use super::offset::Offset;

///Half open range for memory index values
/// Exlusive of min. 
#[derive(Clone, Copy)]
pub struct DecreasingRange
{
    pub max: usize,
    pub cur: usize,
    pub min: usize
}

impl Iterator for DecreasingRange
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur > self.min
        {
            let val = self.cur - 1;
            Some(mem::replace(&mut self.cur, val))
        }
        else {
            None
        }
    }
}

impl DecreasingRange
{
    pub fn new(max: usize, min: usize) -> Self
    {
        DecreasingRange
        {
            max: max,
            cur: max,
            min: min
        }
    }

    pub fn reset(&mut self)
    {
        self.cur = self.max
    }
}

/// Half open range for 2d memory extents row values
/// Exclusive of max_row. 
/// mimics min_row..max_row
#[derive(Clone, Copy)]
pub struct IncreasingRange
{
    pub min: usize,
    pub cur: usize,
    pub max: usize,
}

impl Iterator for IncreasingRange
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.max
        {
            let n = self.cur + 1;
            Some(mem::replace(&mut self.cur, n))
        }
        else {
            None
        }
    }
}

impl IncreasingRange
{
    pub fn new(min: usize, max: usize) -> Self
    {
        IncreasingRange
        {
            min: min,
            cur: min,
            max: max
        }
    }

    pub fn reset(&mut self)
    {
        self.cur = self.min;
    }
}

union RangeUnion
{
    pub pos: IncreasingRange,
    pub neg: DecreasingRange
}

pub struct MemoryIndexRange
{
    range: RangeUnion,
    direction: Offset
}

impl Iterator for MemoryIndexRange
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {

        match self.direction
        {
            Offset::Pos(_) =>
            {
                unsafe{
                    return self.range.pos.next();
                }
            },
            Offset::Neg(_) =>
            {
                unsafe{
                    return self.range.neg.next();
                }
            }
        }
    }
}

impl MemoryIndexRange
{

    pub fn cur(&self) -> usize
    {
        match self.direction
        {
            Offset::Pos(_) => 
            {
                unsafe {
                    self.range.pos.cur
                }
            },
            Offset::Neg(_) =>
            {
                unsafe 
                {
                    self.range.neg.cur
                }
            }
        }
    }

    pub fn new_positive(pos_range: IncreasingRange) -> Self
    {
        MemoryIndexRange
        {
            range: RangeUnion
            {
                pos: pos_range
            },
            direction: Offset::Pos(1)
        }
    }

    pub fn new_negative(neg_range: DecreasingRange) -> Self
    {
        MemoryIndexRange
        {
            range: RangeUnion
            {
                neg: neg_range
            },
            direction: Offset::Neg(1)
        }
    }

    pub fn reset_flip(&mut self)
    {
        match self.direction
        {
            Offset::Pos(_) =>
            {
                unsafe 
                {
                    let new_range: DecreasingRange = DecreasingRange::new(
                        self.range.pos.max, 
                        self.range.pos.min);

                    self.range = RangeUnion
                    {
                        neg: new_range
                    };

                    self.direction = self.direction.flip();
                }
            },
            Offset::Neg(_) =>
            {
                unsafe 
                {
                    let new_range: IncreasingRange = IncreasingRange::new(
                        self.range.neg.min,
                        self.range.neg.max); 

                    self.range = RangeUnion
                    {
                        pos: new_range
                    };

                    self.direction = self.direction.flip();
                }
            }
        }
    }
}