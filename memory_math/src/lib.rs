pub mod memory_extents2d;
pub mod memory_index2d;
pub mod memory_index_range;
pub mod memory_iter;
pub mod memory_range;
pub mod memory_vect2d;
pub mod offset;
pub mod offset_coordinate;
pub mod offset_iter;
pub mod offset_vect2d;
pub mod vector_math;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
