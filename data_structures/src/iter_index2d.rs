use memory_math::{memory_extents2d::HasMemExtents2D, memory_range::LeftToRightRead};

pub trait CanIterIndex2D: HasMemExtents2D {
    fn iter_index2d(&self) -> Result<LeftToRightRead, &'static str> {
        match self.get_extents() {
            Ok(ext) => LeftToRightRead::try_from(ext),
            Err(msg) => Err(msg),
        }
    }
}

