use super::{memory_index2d::MemIndex2D, offset::Offset};

pub trait HasCurMemIndex
{
    fn get_cur_mem_index(&self) -> MemIndex2D;
}

pub trait IterateWithMemIndex<T> : Iterator<Item = T> + HasCurMemIndex + Sized
{
    fn iterate_with_mem_index(self) -> IndexEnumerate<Self>
    {
        IndexEnumerate::new(self)
    }
}

impl<T, I: Iterator<Item = T> + HasCurMemIndex> IterateWithMemIndex<T> for I {}

pub struct IndexEnumerate<I>
    where I: HasCurMemIndex + Iterator
{
    iter: I
}

impl<I> Iterator for IndexEnumerate<I>
where I: HasCurMemIndex + Iterator
{
    type Item = (MemIndex2D, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()
        {
            Some(val) => Some((self.iter.get_cur_mem_index(), val)),
            None => None
        }
    }
}

impl<I> IndexEnumerate<I> 
    where I: HasCurMemIndex + Iterator
{
    pub fn new(iter: I) -> IndexEnumerate<I> {
        IndexEnumerate { iter }
    }
}

pub struct LineRange
{
    pub start_index: MemIndex2D,
    pub cur_index: MemIndex2D,
    pub end_index: MemIndex2D,
    dx: i32,
    dy: i32,
    sx: Offset,
    sy: Offset,
    err: i32
}

impl LineRange
{
    pub fn new(start_index: MemIndex2D, end_index: MemIndex2D) -> Self
    {
        let mut x0 = start_index.col as i32;
        let mut y0 = start_index.row as i32;
        let x1 = end_index.col as i32;
        let y1 = end_index.row as i32;

        if x1.abs_diff(x0) <= 1 && y1.abs_diff(y0) <= 1
        {
            return LineRange
            {
                start_index,
                cur_index: end_index,
                end_index,
                dx: 0,
                dy: 0,
                sx: Offset::zero(),
                sy: Offset::zero(),
                err: 0
            }
        }

        let dx: i32 = i32::abs(x1 - x0);
        let dy: i32 = -i32::abs(y1 - y0);

        let sx: Offset = if x0 < x1 {Offset::Pos(1)} else {Offset::Neg(1)};
        let sy: Offset = if y0 < y1 {Offset::Pos(1)} else {Offset::Neg(1)};
        let err: i32 = dx + dy; // error value e_xy

        //x0 and y0 are just mem index values
        let cur_index: MemIndex2D = MemIndex2D::try_from((y0, x0)).ok().unwrap();

        LineRange
        {
            start_index,
            cur_index,
            end_index,
            dx,
            dy,
            sx,
            sy,
            err
        }
    }
}

impl Iterator for LineRange
{
    type Item = MemIndex2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index == self.end_index
        {
            return None;
        }

        let e2 = 2 * self.err;
        if e2 >= self.dy 
        { 
            self.err += self.dy; 

            match self.cur_index.col + self.sx
            {
                Some(val) =>
                {
                    self.cur_index.col = val;
                },
                None => 
                {
                    return None;
                }
            }
        } // e_xy+e_x > 0
        
        if e2 <= self.dx 
        { 
            self.err += self.dx; 

            match self.cur_index.row + self.sy
            {
                Some(p_val) => {self.cur_index.row = p_val;},
                None => {return None;}
            }
        } // e_xy+e_y < 0

        return Some(self.cur_index);
    }
}