use std::ops::Sub;

use super::memory_index2d::MemIndex2D;



/// In memory space there is a very clear set of comparisons and operations
/// which may be made, MemoryVect2d (mv) always lie in the first quadrant (x+, y+) and 
/// always have integer components. so some operations have a much more direct intepretation 
/// than in the euclidian plane. For example mv_a < mv_b has an exact meaning whereas 
/// v_a < v_b doesn't neccesarily have an exact meaning and needs a little more context. 
#[derive(Clone, Copy)]
pub struct MemVect2D{
    pub row: usize,
    pub col: usize
}

impl From<MemIndex2D> for MemVect2D{
    fn from(value: MemIndex2D) -> Self {
        MemVect2D
        {
            row: value.row,
            col: value.col
        }
    }
}

impl PartialEq for MemVect2D
{
    fn eq(&self, other: &Self) -> bool
    {
        return self.row == other.row && self.col == other.col;
    }
}
impl PartialOrd for MemVect2D
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.row.partial_cmp(&other.row) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.col.partial_cmp(&other.col)
    }
}

impl Sub for MemVect2D
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

        Some(MemVect2D{row: row_component, col: col_component})
    }
}

impl MemVect2D
{
    pub fn new(row: usize, col: usize) -> Self
    {
        MemVect2D
        {
            row,
            col
        }
    }

    pub fn row_vect(magnitude: usize) -> Self
    {
        MemVect2D
        {
            row: magnitude,
            col: 0
        }
    }

    pub fn col_vect(magnitude: usize) -> Self
    {
        MemVect2D
        {
            row: 0,
            col: magnitude
        }
    }
}