use core::fmt;
use std::ops::{Add, Mul, Sub};

use super::memory_index2d::MemIndex2D;
use super::offset::Offset;


#[derive(Clone, Copy)]
pub struct OffsetVect2D
{
    pub row: Offset,
    pub col: Offset
}

impl fmt::Display for OffsetVect2D
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl Add<Self> for OffsetVect2D
{
    type Output = Self;

    fn add(self, other: Self) -> Self
    {
        OffsetVect2D::new(self.row + other.row, self.col + other.col)
    }
}

impl Add<MemIndex2D> for OffsetVect2D
{
    type Output = OffsetVect2D;

    fn add(self, rhs: MemIndex2D) -> Self::Output {
        OffsetVect2D
        {
            row: self.row + rhs.row,
            col: self.col + rhs.col
        }
    }
}

impl Sub for OffsetVect2D
{
    type Output = Self;

    fn sub(self, other: Self) -> Self
    {
        OffsetVect2D::new(self.row - other.row, self.col - other.col)
    }
}

impl Mul<f32> for OffsetVect2D{
    type Output = Option<Self>;

    fn mul(self, rhs: f32) -> Self::Output {
        let r_val = match self.row * rhs
        {
            Some(r) => r,
            None => {return None;}
        };

        let c_val = match self.col * rhs
        {
            Some(c) => c,
            None => {return None;}
        };

        Some(OffsetVect2D{row: r_val, col: c_val})
    }
}

impl Mul<i8> for OffsetVect2D{
    type Output = Self;
    
    fn mul(self, rhs: i8) -> Self::Output {
        OffsetVect2D::new(self.row * rhs, self.col * rhs)
    }

    
}

impl From<MemIndex2D> for OffsetVect2D
{
    fn from(value: MemIndex2D) -> Self {
        OffsetVect2D
        {
            row: Offset::Pos(value.row),
            col: Offset::Pos(value.col)
        }
    }
}

impl From<(usize, usize)> for OffsetVect2D
{
    fn from(value: (usize, usize)) -> Self {
        OffsetVect2D
        {
            row: Offset::Pos(value.0),
            col: Offset::Pos(value.1)
        }
    }
}

impl OffsetVect2D
{

    pub fn zero() -> Self
    {
        OffsetVect2D
        {
            row: Offset::zero(),
            col: Offset::zero()
        }
    }

    pub fn length(&self) -> f32
    {
        Offset::sqrt(self.row * self.row + self.col * self.col)
    }

    pub fn new_from_index(index: MemIndex2D) -> Self
    {
        OffsetVect2D
        {
            row: Offset::Pos(index.row),
            col: Offset::Pos(index.col)
        }
    }

    pub fn new_signed(row: isize, column: isize) -> Self
    {
        OffsetVect2D
        {
            row: Offset::from(row),
            col: Offset::from(column)
        }
    }

    pub fn new(row: Offset, column: Offset) -> Self
    {
        OffsetVect2D
        {
            row: row,
            col: column
        }
    }

    
}
