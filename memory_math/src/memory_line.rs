use crate::memory_index2d::MemIndex2D;

pub struct MemLine2D
{
    pub min: MemIndex2D,
    pub max: MemIndex2D
}

impl MemLine2D
{
    pub fn new(min: MemIndex2D, max: MemIndex2D) -> Self
    {
        MemLine2D { min, max }
    }

    fn plot_line_high(min: MemIndex2D, max: MemIndex2D) -> Vec<MemIndex2D>
    {
        let mut dx: i32 = max.col as i32 - min.col as i32;
        let dy: i32 = max.row as i32 - min.row as i32;
        let mut xi: i32 = 1;
        if dx < 0
        {
            xi = -1;
            dx = -dx;
        }

        let mut d: i32 = (2 * dx) - dy;
        let mut x = min.col;

        let mut indexes: Vec<MemIndex2D> = Vec::new();

        for row in min.row..max.row
        {
            indexes.push(MemIndex2D::new(row, x));

            if d > 0
            {
                if xi >= 0
                {
                    x = x + xi as usize;
                }
                else {
                    x = x.wrapping_sub((-xi) as usize);
                }
                d = d + (2 * (dx - dy));
            }
            else
            {
                d = d + 2*dx;
            }
        }

        indexes
    }

    fn plot_line_low(min: MemIndex2D, max: MemIndex2D) -> Vec<MemIndex2D>
    {
        let dx: i32 = max.col as i32 - min.col as i32;
        let mut dy: i32 = max.row as i32 - min.row as i32;
        let mut yi: i32 = 1;
        if dy < 0
        {
            yi = -1;
            dy = -dy;
        }

        let mut d: i32 = (2 * dy) - dx;
        let mut y = min.row;

        let mut indexes: Vec<MemIndex2D> = Vec::new(); 
        for col in min.col..max.col
        {
            indexes.push(MemIndex2D { row: y, col });
            
            if d > 0
            {
                if yi >= 0
                {
                    y = y + yi as usize;
                }
                else {
                    y = y.wrapping_sub((-yi) as usize);
                }
                
                d = d + (2 * (dy - dx));
            }
            else
            {
                d = d + 2*dy;
            }
        }

        return indexes;
    }

    fn plot_line(min: MemIndex2D, max: MemIndex2D) -> Vec<MemIndex2D>
    {
        if max.row.abs_diff(min.row) < max.col.abs_diff(min.col)
        {
            if min.col > max.col
            {
                Self::plot_line_low(max, min)
            }
            else {
                Self::plot_line_low(min, max)
            }
        }
        else {
            if min.row > max.row
            {
                Self::plot_line_high(max, min)
            }
            else {
                Self::plot_line_high(min, max)
            }
        }
    }

    pub fn line_indexes(&self) -> Vec<MemIndex2D>
    {
        if self.min.row == self.max.row //horizontal line
        {
            let mut indexes: Vec<MemIndex2D> = Vec::with_capacity(self.max.col - self.min.col);
            for col in self.min.col..self.max.col
            {
                indexes.push(MemIndex2D { row: self.min.row, col });
            }

            return indexes;
        }
        else if self.min.col == self.max.col //vertical line
        {
            let mut indexes: Vec<MemIndex2D> = Vec::with_capacity(self.max.row - self.min.row);
            for row in self.min.row..self.max.row
            {
                indexes.push(MemIndex2D { row, col: self.min.col });
            }

            return indexes;
        }

        Self::plot_line(self.min, self.max)
    }
}