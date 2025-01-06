use super::{offset::Offset, offset_vect2d::OffsetVect2D};


#[derive(Clone, Copy)]
pub struct OffsetCoordinate2D
{
    pub row: Offset,
    pub col: Offset
}

impl std::fmt::Display for OffsetCoordinate2D
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl From<OffsetVect2D> for OffsetCoordinate2D
{
    fn from(value: OffsetVect2D) -> Self {
        OffsetCoordinate2D
        {
            row: value.row,
            col: value.col
        }
    }
}

impl OffsetCoordinate2D
{
    pub fn new(row: Offset, col: Offset) -> Self
    {
        OffsetCoordinate2D
        {
            row,
            col
        }
    }
}