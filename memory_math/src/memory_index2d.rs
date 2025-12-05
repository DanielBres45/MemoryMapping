use std::num::TryFromIntError;
use std::ops::Sub;
use std::{fmt, ops::Add};
use std::cmp::Ordering;
use super::memory_offset2d::MemOffset2D;


//TODO: Write lexicographic comparer
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

impl Eq for MemIndex2D{}

impl PartialOrd for MemIndex2D{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        let col_compare = self.col.cmp(&other.col);
        if col_compare != Ordering::Equal
        {
            return Some(col_compare);
        }
        
        return self.row.partial_cmp(&other.row);
    }
}

impl Ord for MemIndex2D{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<MemOffset2D> for MemIndex2D
{
    type Error = &'static str;

    fn try_from(value: MemOffset2D) -> Result<Self, Self::Error> {

        if value.row < 0 || value.col < 0
        {
            return Err("MemOffset2D must be positive");
        }

        Ok(MemIndex2D::new(value.row as usize, value.col as usize))
    }
}

impl Sub for MemIndex2D{
    type Output = MemOffset2D;

    fn sub(self, rhs: Self) -> Self::Output {
        let row: isize = self.row as isize - rhs.row as isize;
        let col: isize = self.col as isize - rhs.col as isize;

        MemOffset2D::new(row, col)
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
impl Add<MemOffset2D> for MemIndex2D{
    type Output = Option<MemIndex2D>;

    fn add(self, rhs: MemOffset2D) -> Self::Output {

        let row: usize = self.row.checked_add_signed(rhs.row)?;
        let col: usize = self.col.checked_add_signed(rhs.col)?;

        Some(MemIndex2D{
            row,
            col
        })
    }
}


impl Sub<MemOffset2D> for MemIndex2D
{
    type Output = Option<MemIndex2D>;

    fn sub(self, rhs: MemOffset2D) -> Self::Output {
        let flipped = -rhs;
        self + flipped
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