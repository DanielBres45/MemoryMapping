use std::{usize, vec::IntoIter};

use memory_math::{memory_index2d::MemIndex2D, memory_iter::HasCurMemIndex};

pub struct Vec2DIntoIter<T> {
    vec: IntoIter<T>,
    cur_col: usize,
    cur_row: usize,
    vec_width: usize,
}

impl<T> HasCurMemIndex for Vec2DIntoIter<T> {
    fn get_cur_mem_index(&self) -> MemIndex2D {
        MemIndex2D::new(self.cur_row, self.cur_col)
    }
}

impl<T> Iterator for Vec2DIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vec.next() {
            Some(val) => {
                self.increment_col();

                Some(val)
            }
            None => None,
        }
    }
}

impl<T> Vec2DIntoIter<T> {
    pub fn new(vec: IntoIter<T>, vec_width: usize) -> Self {
        Vec2DIntoIter {
            vec,
            cur_row: 0,
            cur_col: usize::MAX,
            vec_width,
        }
    }

    fn increment_col(&mut self) {
        self.cur_col = self.cur_col.wrapping_add(1);

        if self.cur_col == self.vec_width {
            self.cur_col = 0;
            self.cur_row += 1;
        }
    }
}

