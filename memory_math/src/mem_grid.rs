use crate::memory_index2d::MemIndex2D;
use crate::size_2d::{HasSize2D, Size2D};
use crate::memory_span2d::MemSpan2D;


pub trait MemoryGrid : HasSize2D
{

    fn grid_row_count(&self) -> usize;
    fn grid_column_count(&self) -> usize;
    #[inline]
    fn grid_size(&self) -> Size2D {
        Size2D::new(self.grid_column_count(), self.grid_row_count())
    }

    #[inline]
    fn grid_index_in_bounds(&self, grid_index: &GridIndex) -> bool
    {
        self.grid_size().index2d_in_bounds(&grid_index.index2d())
    }

    fn row_index_to_grid_row_index(&self, row_index: usize) -> Option<usize>;
    fn grid_row_index_to_row_index(&self, grid_row_index: usize) -> Option<usize>;
    fn column_index_to_grid_column_index(&self, column_index: usize) -> Option<usize>;
    fn grid_column_index_to_column_index(&self, grid_column_index: usize) -> Option<usize>;

    fn index2d_to_grid_index(&self, index2d: &MemIndex2D) -> Option<GridIndex>
    {
        let row_index = self.row_index_to_grid_row_index(index2d.row)?;
        let column_index = self.column_index_to_grid_column_index(index2d.col)?;

        Some(GridIndex::new(row_index, column_index))
    }
    fn index2d_relative_to_grid(&self, cell_index2d: &MemIndex2D, grid_index: &GridIndex) -> Option<MemIndex2D>;
    fn grid_index_to_index2d(&self, grid_index: &GridIndex) -> Option<MemIndex2D>;

    fn grid_index_span2d(&self, grid_index: &GridIndex) -> Option<MemSpan2D>
    {
        let min_index2d = self.grid_index_to_index2d(grid_index)?;

        let max_row: usize = self.grid_row_index_to_row_index(grid_index.row() + 1).unwrap_or(self.max_row()?);
        let max_col: usize = self.grid_column_index_to_column_index(grid_index.col() + 1).unwrap_or(self.max_col()?);

        let max_index2d = MemIndex2D::new(max_row, max_col);
        Some(MemSpan2D::new_from_index2d(min_index2d, max_index2d))
    }

    fn grid_intersections(&self, span2d: &MemSpan2D) -> Option<(GridRange2D, Vec<GridIntersection>)>
    {
        let mut intersections: Vec<GridIntersection> = Vec::new();

        let min_grid_index: GridIndex = self.index2d_to_grid_index(&span2d.min_index2d())?;
        let max_grid_index: GridIndex = self.index2d_to_grid_index(&span2d.max_index2d()?)?;

        for row in min_grid_index.row()..=max_grid_index.row()
        {
            for col in min_grid_index.col()..=max_grid_index.col()
            {
                let grid_index = GridIndex::new(row, col);
                let cur_grid_span =  self.grid_index_span2d(&grid_index)?; //if this is none, grid is malformed

                if let Some(intersection) = cur_grid_span.intersect(span2d)
                {
                    intersections.push(GridIntersection{grid_index, intersection});
                }
            }
        }

        Some((GridRange2D(MemSpan2D::new_from_index2d(min_grid_index.index2d(), MemIndex2D{row: max_grid_index.row() + 1, col: max_grid_index.col() + 1}) ), intersections))
    }
}

pub struct MemGrid2D
{
    pub size: Size2D,
    pub row_offset: usize,
    pub column_offset: usize
}

pub struct GridIndex(pub MemIndex2D);

impl GridIndex
{
    pub fn new(row: usize, col: usize) -> Self {
        GridIndex(MemIndex2D::new(row, col))
    }

    #[inline]
    pub fn row(&self) -> usize {
        self.0.row
    }

    #[inline]
    pub fn col(&self) -> usize {
        self.0.col
    }

    #[inline]
    pub fn index2d(&self) -> MemIndex2D {
        self.0
    }
}

pub struct GridIntersection {
    pub grid_index: GridIndex,
    pub intersection: MemSpan2D,
}

pub struct GridRange2D(pub MemSpan2D);

impl HasSize2D for MemGrid2D
{
    fn row_count(&self) -> usize {
        self.size.row_count
    }

    fn column_count(&self) -> usize {
        self.size.column_count
    }

    fn size(&self) -> Size2D {
        self.size
    }
}

impl MemoryGrid for MemGrid2D
{

    #[inline]
    fn grid_row_count(&self) -> usize {
        self.size.row_count / self.row_offset
    }

    #[inline]
    fn grid_column_count(&self) -> usize {
        self.size.column_count / self.column_offset
    }

    fn grid_row_index_to_row_index(&self, grid_row_index: usize) -> Option<usize>
    {
        if grid_row_index >= self.grid_row_count()
        {
            return None;
        }

        Some(grid_row_index * self.row_offset)
    }

    fn grid_column_index_to_column_index(&self, grid_column_index: usize) -> Option<usize>
    {
        if grid_column_index >= self.grid_column_count()
        {
            return None;
        }

        Some(grid_column_index * self.column_offset)
    }

    #[inline]
    fn row_index_to_grid_row_index(&self, row_index: usize) -> Option<usize>
    {
        if row_index >= self.row_count()
        {
            return None;
        }

        Some(row_index / self.row_offset)
    }

    #[inline]
    fn column_index_to_grid_column_index(&self, column_index: usize) -> Option<usize>
    {
        if column_index >= self.column_count()
        {
            return None;
        }

        Some(column_index / self.column_offset)
    }

    fn index2d_relative_to_grid(&self, cell_index2d: &MemIndex2D, grid_index: &GridIndex) -> Option<MemIndex2D> {

        let grid_row: usize = self.grid_row_index_to_row_index(grid_index.row())?;
        let grid_col: usize = self.grid_column_index_to_column_index(grid_index.col())?;

        let row: usize = cell_index2d.row.checked_sub(grid_row)?;
        let col: usize = cell_index2d.col.checked_sub(grid_col)?;

        Some(MemIndex2D{row, col})
    }

    fn grid_index_to_index2d(&self, grid_index: &GridIndex) -> Option<MemIndex2D> {
        if !self.grid_index_in_bounds(&grid_index)
        {
            return None;
        }

        Some(MemIndex2D::new(
            grid_index.row() * self.row_offset,
            grid_index.col() * self.column_offset
        ))
    }



}

impl MemGrid2D
{
    pub fn new(size: Size2D, row_offset: usize, column_offset: usize) -> Self {
        MemGrid2D {
            size,
            row_offset,
            column_offset
        }
    }



}

pub struct NonUniformMemGrid2D
{
    pub size: Size2D,
    pub row_offsets: Vec<usize>,
    pub column_offsets: Vec<usize>
}

impl HasSize2D for NonUniformMemGrid2D
{
    fn row_count(&self) -> usize {
        self.size.row_count
    }

    fn column_count(&self) -> usize {
        self.size.column_count
    }

    fn size(&self) -> Size2D {
        self.size
    }
}

impl MemoryGrid for NonUniformMemGrid2D
{
    #[inline]
    fn grid_row_count(&self) -> usize {
        self.row_offsets.len()
    }

    #[inline]
    fn grid_column_count(&self) -> usize {
        self.column_offsets.len()
    }

    //binary search on row offsets to find the tile row which
    //contains the current row index
    fn row_index_to_grid_row_index(&self, cell_row_index: usize) -> Option<usize>
    {
        if cell_row_index >= self.size.row_count()
        {
            return None;
        }

        Some(Self::index_search(
            &self.row_offsets,
            cell_row_index
        ))
    }

    fn grid_row_index_to_row_index(&self, grid_row_index: usize) -> Option<usize>
    {

        if grid_row_index >= self.grid_row_count()
        {
            return None;
        }
        else if grid_row_index == 0
        {
            return Some(0);
        }
        else
        {
            Some(self.row_offsets[grid_row_index - 1])
        }

    }

    fn column_index_to_grid_column_index(&self, cell_column_index: usize) -> Option<usize>
    {
        if cell_column_index >= self.size.column_count()
        {
            return None;
        }

        Some(Self::index_search(
            &self.column_offsets,
            cell_column_index
        ))
    }

    fn grid_column_index_to_column_index(&self, grid_column_index: usize) -> Option<usize> {
        if grid_column_index >= self.grid_column_count()
        {
            return None;
        }
        else if grid_column_index == 0
        {
            return Some(0);
        }
        else
        {
            Some(self.row_offsets[grid_column_index - 1])
        }
    }

    fn index2d_relative_to_grid(&self, cell_index2d: &MemIndex2D, grid_index: &GridIndex) -> Option<MemIndex2D>
    {
        let row_offset: usize = match grid_index.0.row.checked_sub(1) { Some(i) => self.row_offsets[i], None => 0};
        let column_offset: usize = match grid_index.0.col.checked_sub(1) { Some(i) => self.column_offsets[i], None => 0};

        let row: usize = cell_index2d.row.checked_sub(row_offset)?;
        let col: usize = cell_index2d.col.checked_sub(column_offset)?;

        Some(MemIndex2D{row, col})
    }

    fn grid_index_to_index2d(&self, grid_index: &GridIndex) -> Option<MemIndex2D> {
        if !self.grid_index_in_bounds(&grid_index)
        {
            return None;
        }

        let row_offset: usize = self.row_offsets[grid_index.0.row];
        let column_offset: usize = self.column_offsets[grid_index.0.col];
        Some(MemIndex2D::new(row_offset, column_offset))
    }
}

impl NonUniformMemGrid2D
{
    pub fn new(size: Size2D, row_offsets: Vec<usize>, column_offsets: Vec<usize>) -> Self {
        NonUniformMemGrid2D {
            size,
            row_offsets,
            column_offsets
        }
    }

    fn index_search(indexes: &Vec<usize>, search_for: usize) -> usize
    {
        match indexes.binary_search(&search_for) {
            Ok(index) => index + 1,
            Err(index) => index
        }
    }

    //binary search on row offsets to find the tile row which
    //contains the current row index
    fn row_index_to_tile_row_index(&self, cell_row_index: usize) -> Option<usize>
    {
        if cell_row_index >= self.size.row_count()
        {
            return None;
        }

        Some(Self::index_search(
            &self.row_offsets,
            cell_row_index
        ))
    }

    fn column_index_to_tile_column_index(&self, cell_column_index: usize) -> Option<usize>
    {
        if cell_column_index >= self.size.column_count()
        {
            return None;
        }

        Some(Self::index_search(
            &self.column_offsets,
            cell_column_index
        ))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_grid2d()
    {
        let size = Size2D::new(10, 10);
        let grid = MemGrid2D::new(size, 2, 2);
        let index2d = MemIndex2D::new(4, 4);

        let grid_index = grid.index2d_to_grid_index(&index2d).unwrap();
        assert_eq!(grid_index.row(), 2);
        assert_eq!(grid_index.col(), 2);

        let index_relative_to_grid = grid.index2d_relative_to_grid(&index2d, &grid_index).unwrap();
        assert_eq!(0, index_relative_to_grid.row);
        assert_eq!(0, index_relative_to_grid.col);

        assert!(grid.index2d_to_grid_index(&MemIndex2D::new(10, 10)).is_none());
    }

    #[test]
    fn test_grid_intersections_all_grid2d()
    {
        let size: Size2D = Size2D::new(30,30);
        let row_offset: usize = 10;
        let column_offset: usize = 10;
        let grid = MemGrid2D::new(size, row_offset, column_offset);

        let test_range2d: MemSpan2D = MemSpan2D::new_from_index2d(MemIndex2D{row: 5, col: 5}, MemIndex2D{row: 25, col: 25});
        let (grid_range, intersections) = grid.grid_intersections(&test_range2d).unwrap();

        let expected_grid_range: GridRange2D = GridRange2D(MemSpan2D::new_from_usize(0,0,3,3));
        assert_eq!(expected_grid_range.0, grid_range.0);

        assert_eq!(intersections.len(), 9);

        let expected_indexes: Vec<MemIndex2D> = vec![
            MemIndex2D::origin(),
            MemIndex2D{row: 0, col: 1},
            MemIndex2D{row: 0, col: 2},
            MemIndex2D{row: 1, col: 0},
            MemIndex2D{row: 1, col: 1},
            MemIndex2D{row: 1, col: 2},
            MemIndex2D{row: 2, col: 0},
            MemIndex2D{row: 2, col: 1},
            MemIndex2D{row: 2, col: 2},
        ];

        let actual_indexes: Vec<MemIndex2D> = intersections.iter().map(|i| i.grid_index.0).collect();
        assert_eq!(expected_indexes, actual_indexes);
    }

    #[test]
    fn test_non_uniform_mem_grid2d()
    {
        let size = Size2D::new(10, 10);
        let row_offsets = vec![2, 4, 6, 8];
        let column_offsets = vec![2, 4, 6, 8];
        let grid = NonUniformMemGrid2D::new(size, row_offsets, column_offsets);
        let index2d = MemIndex2D::new(4, 4);

        let grid_index = grid.index2d_to_grid_index(&index2d).unwrap();

        assert_eq!(MemIndex2D{row: 2, col: 2}, grid_index.0);

        let index_relative_to_grid = grid.index2d_relative_to_grid(&index2d, &grid_index).unwrap();
        assert_eq!(MemIndex2D{row: 0, col: 0}, index_relative_to_grid);

        assert!(grid.index2d_relative_to_grid(&MemIndex2D::new(1, 1), &GridIndex::new(1, 1)).is_none());
        assert!(grid.index2d_to_grid_index(&MemIndex2D::new(10, 10)).is_none());

        
    }
}