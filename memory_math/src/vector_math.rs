use super::memory_index2d::MemIndex2D;

pub fn bresenham_line(start_coords: MemIndex2D, end_coords: MemIndex2D) -> Vec<MemIndex2D> {
    let mut points: Vec<MemIndex2D> = Vec::new();

    let mut x0 = start_coords.col as i32;
    let mut y0 = start_coords.row as i32;
    let x1 = end_coords.col as i32;
    let y1 = end_coords.row as i32;

    if x1.abs_diff(x0) <= 1 && y1.abs_diff(y0) <= 1 {
        return vec![end_coords];
    }

    let dx = i32::abs(x1 - x0);
    let dy = -i32::abs(y1 - y0);
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy; // error value e_xy

    loop {
        let new_index = MemIndex2D::try_from((y0, x0));

        match new_index {
            Ok(ind) => points.push(ind),
            Err(_) => {
                break;
            }
        }

        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        } // e_xy+e_x > 0
        if e2 <= dx {
            err += dx;
            y0 += sy;
        } // e_xy+e_y < 0
    }

    points
}

