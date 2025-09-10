use std::num::TryFromIntError;
use std::ops::Sub;
use std::{fmt, ops::Add};

use super::memory_vect2d::MemVect2D;
use super::offset_coordinate::OffsetCoordinate2D;
use super::offset_vect2d::OffsetVect2D;
use super::offset::Offset;

#[derive(Copy, Clone)]
pub struct MemIndex2D
{
    pub row: usize,
    pub col: usize
}

impl fmt::Display for MemIndex2D
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl fmt::Debug for MemIndex2D
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemIndex2D").field("row", &self.row).field("col", &self.col).finish()
    }
}

impl PartialEq for MemIndex2D{
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

impl PartialOrd for MemIndex2D{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        let row_compare = self.row.partial_cmp(&other.row).unwrap(); //usize comparison always exists

        match row_compare
        {
            std::cmp::Ordering::Less => Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Equal => self.col.partial_cmp(&other.col),
            std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Greater)
        }
    }
}

impl Add<MemVect2D> for MemIndex2D{
    type Output = MemIndex2D;

    fn add(self, rhs: MemVect2D) -> Self::Output{
        MemIndex2D
        {
            row: self.row + rhs.row,
            col: self.col + rhs.col
        }
    }
}



impl TryFrom<OffsetCoordinate2D> for MemIndex2D
{
    type Error = &'static str;

    fn try_from(value: OffsetCoordinate2D) -> Result<Self, Self::Error> {
        match (value.row, value.col)
        {
            (Offset::Pos(r_val), Offset::Pos(c_val)) => Ok(MemIndex2D::new(r_val, c_val)),
            _ => Err("row and col must be positive")
        }
    }
}

impl From<MemVect2D> for MemIndex2D{
    fn from(value: MemVect2D) -> Self {
        MemIndex2D{
            row: value.row, 
            col: value.col
        }
    }
}

impl Sub for MemIndex2D{
    type Output = OffsetVect2D;

    fn sub(self, rhs: Self) -> Self::Output {
        OffsetVect2D::from(self) - OffsetVect2D::from(rhs)
    }
}

impl Sub<MemVect2D> for MemIndex2D{
    type Output = Option<Self>;

    fn sub(self, rhs: MemVect2D) -> Self::Output{
        match MemVect2D::from(self) - rhs
        {
            Some(v_val) => Some(Self::from(v_val)),
            None => None
        }
    }
}

impl TryFrom<(f32, f32)> for MemIndex2D
{
    type Error = String;

    fn try_from(value: (f32, f32)) -> Result<Self, Self::Error> {
        let (row, col) = value;

        if !f32::is_finite(row)
        {
            return Err(format!("row value: {} is not a number", row));
        }

        let row_val: usize = match usize::try_from(row.round() as i32) {
            Ok(v) => v,
            Err(_) => {return Err(format!("row is not a usize: {}", row));}
        };

        if !f32::is_finite(col)
        {
            return Err(format!("col value: {} is not a number", col));
        }

        let col_val: usize = match usize::try_from(col.round() as i32)
        {
            Ok(c) => c,
            Err(_) => {return Err(format!("col is not a usize: {}", col));}
        };

        Ok(MemIndex2D { row: row_val, col: col_val })

    }
}

impl TryFrom<(i32, i32)> for MemIndex2D
{
    type Error = TryFromIntError;

    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        
        let (row, col) = value;
        
        let row_val = match usize::try_from(row)
        {
            Ok(r) => r,
            Err(try_err) => 
            {
                return Err(try_err);
            }
        };

        let col_val = match usize::try_from(col)
        {
            Ok(c) => c,
            Err(try_err) => 
            {
                return Err(try_err);
            }
        };

        Ok(MemIndex2D{
            row: row_val,
            col: col_val
        })
    }
}

///Attempt to add a MemIndex2D with an OffsetVect2D, if possible.
/// This is possible only if the OffsetVect2D lies entriely within the first quadrant,
/// which then proceeds to act like addition with a MemVect2D. 
/// return None if impossible.
impl Add<OffsetVect2D> for MemIndex2D{
    type Output = Option<MemIndex2D>;

    fn add(self, rhs: OffsetVect2D) -> Self::Output {
        match(rhs.row + self.row, rhs.col + self.col)
        {
            (Offset::Pos(r), Offset::Pos(c)) =>
            {
                Some(MemIndex2D::new(r, c))
            }
            _ => None
        }
    }
}

impl Sub<OffsetVect2D> for MemIndex2D
{
    type Output = OffsetVect2D;

    fn sub(self, rhs: OffsetVect2D) -> Self::Output {
        OffsetVect2D
        {
            row: self.row - rhs.row,
            col: self.col - rhs.col
        }
    }
}

impl MemIndex2D
{
    pub fn new(row: usize, col: usize) -> Self{
        MemIndex2D { row, col}
    }

    pub fn origin() -> Self{
        MemIndex2D { row: 0, col: 0 }
    }
}