pub mod iter_index2d;
pub mod option_vec2d;
pub mod tile_map;
pub mod vec2d;
pub mod vec2d_iter;
pub mod vec_2d;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
