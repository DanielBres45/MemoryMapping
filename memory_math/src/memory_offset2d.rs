use std::ops::{Add, Neg, Sub};

use super::memory_index2d::MemIndex2D;



#[derive(Clone, Copy)]
pub struct MemOffset2D {
    pub row: isize,
    pub col: isize
}

impl From<MemIndex2D> for MemOffset2D {
    fn from(value: MemIndex2D) -> Self {
        MemOffset2D
        {
            row: value.row as isize,
            col: value.col as isize
        }
    }
}

impl PartialEq for MemOffset2D
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.row == other.row && self.col == other.col;
    }
}
impl PartialOrd for MemOffset2D
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.row.partial_cmp(&other.row) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.col.partial_cmp(&other.col)
    }
}

impl Add<MemIndex2D> for MemOffset2D {

    type Output = Option<MemIndex2D>;
    fn add(self, rhs: MemIndex2D) -> Self::Output {
        rhs + self
    }
}

impl Sub for MemOffset2D
{
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        
        let row_component = match self.row.checked_sub(rhs.row)
        {
            Some(r_val) => r_val,
            None => return None
        };

        let col_component = match self.col.checked_sub(rhs.col)
        {
            Some(c_val) => c_val,
            None => return None
        };

        Some(MemOffset2D {row: row_component, col: col_component})
    }
}

impl Neg for MemOffset2D
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        MemOffset2D {row: -self.row, col: -self.col}
    }
}

impl MemOffset2D
{
    pub fn new(row: isize, col: isize) -> Self
    {
        MemOffset2D
        {
            row,
            col
        }
    }

    pub fn row_offset(magnitude: isize) -> Self
    {
        MemOffset2D
        {
            row: magnitude,
            col: 0
        }
    }

    pub fn col_offset(magnitude: isize) -> Self
    {
        MemOffset2D
        {
            row: 0,
            col: magnitude
        }
    }
}