use super::offset_coordinate::OffsetCoordinate2D;

pub trait HasCurOffsetCoordinate
{
    fn get_cur_offset_coordinate(&self) -> OffsetCoordinate2D;
}


pub trait IterateWithOffsetCoordinate<T> : Iterator<Item = T> + HasCurOffsetCoordinate + Sized
{
    fn iterate_with_offset_coordinate(self) -> OffsetCoordinateEnumerate<Self>
    {
        OffsetCoordinateEnumerate::new(self)
    }
}

impl<T, I: Iterator<Item = T> + HasCurOffsetCoordinate> IterateWithOffsetCoordinate<T> for I {}

pub struct OffsetCoordinateEnumerate<I>
    where I: HasCurOffsetCoordinate + Iterator
{
    iter: I
}

impl<I> Iterator for OffsetCoordinateEnumerate<I>
where I: HasCurOffsetCoordinate + Iterator
{
    type Item = (OffsetCoordinate2D, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()
        {
            Some(val) => Some((self.iter.get_cur_offset_coordinate(), val)),
            None => None
        }
    }
}

impl<I> OffsetCoordinateEnumerate<I> 
    where I: HasCurOffsetCoordinate + Iterator
{
    pub fn new(iter: I) -> OffsetCoordinateEnumerate<I> {
        OffsetCoordinateEnumerate { iter }
    }
}
