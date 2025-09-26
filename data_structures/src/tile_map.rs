use std::{
    clone, fmt,
    fmt::Pointer,
    ops::{Index, IndexMut},
};

use memory_math::{
    memory_span2d::{HasMemSpan2D, MemSpan2D},
    memory_index2d::MemIndex2D,
    memory_offset2d::MemOffset2D,
};
use memory_math::memory_iterators::LinearMemoryIterator;
use memory_math::memory_span::MemSpan;
use crate::vec2d::{MutVec2DMethods, Vec2DMethods, Vec2DSlice};
use super::{vec2d::Vec2D, vec2d_iter::Vec2DIntoIter};
use memory_math::mem_grid::{GridIndex, MemGrid2D, MemoryGrid, NonUniformMemGrid2D};
use memory_math::size_2d::{HasSize2D, Size2D};

pub struct TileMap<T>
{
    tiles: Vec2D<Vec2D<T>>,
    grid: MemGrid2D,
}

impl<T> HasSize2D for TileMap<T>
{
    #[inline]
    fn row_count(&self) -> usize {
        self.grid.row_count()
    }

    #[inline]
    fn column_count(&self) -> usize {
        self.grid.column_count()
    }
}

///Range of tiles in a tilemap

pub struct TileRange2D(pub MemSpan2D);

impl HasSize2D for TileRange2D {
    fn row_count(&self) -> usize {
        self.0.row_count()
    }

    fn column_count(&self) -> usize {
        self.0.column_count()
    }

    fn size(&self) -> Size2D {
        self.0.size()
    }
}


pub struct TileIntersection {
    pub grid_index: GridIndex,
    pub intersection: MemSpan2D,
}

impl TileIntersection {
    pub fn new(grid_index: GridIndex, intersection: MemSpan2D) -> Self {
        TileIntersection {
            grid_index,
            intersection,
        }
    }
}

impl PartialEq for TileIntersection {
    fn eq(&self, other: &Self) -> bool {
        self.grid_index.0 == other.grid_index.0 && self.intersection == other.intersection
    }
}

impl fmt::Display for TileIntersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TileIntersection[grid_index: {}, intersection: {}]",
            self.grid_index.0, self.intersection
        )
    }
}

impl fmt::Debug for TileIntersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileIntersection")
            .field("grid_index", &self.grid_index.0)
            .field("intersection", &self.intersection)
            .finish()
    }
}

impl <T> Index<MemIndex2D> for TileMap<T> {
    type Output = T;

    fn index(&self, index: MemIndex2D) -> &Self::Output {
        let grid_index: GridIndex = self.grid.index2d_to_grid_index(&index).unwrap_or_else(|| panic!("Index out of bounds!"));
        let cell_index: MemIndex2D = self.grid.index2d_relative_to_grid(&index, &grid_index).unwrap_or_else(|| panic!("Index out of bounds!"));
        match self.tiles.get_index2d(grid_index.0) {
            Some(tile) => tile.get_index2d(cell_index).unwrap_or_else(|| panic!("Index out of bounds!")),
            None => panic!("Index out of bounds!"),
        }
    }
}


impl<T> IndexMut<MemIndex2D> for TileMap<T> {
    fn index_mut(&mut self, index: MemIndex2D) -> &mut Self::Output {
        let grid_index: GridIndex = self.grid.index2d_to_grid_index(&index).unwrap_or_else(|| panic!("Index out of bounds!"));
        let cell_index: MemIndex2D = self.grid.index2d_relative_to_grid(&index, &grid_index).unwrap_or_else(|| panic!("Index out of bounds!"));
        match self.tiles.get_mut_index2d(grid_index.0) {
            Some(tile) => tile.get_mut_index2d(cell_index).unwrap_or_else(|| panic!("Index out of bounds!")),
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T> Index<GridIndex> for TileMap<T> {
    type Output = Vec2D<T>;

    fn index(&self, index: GridIndex) -> &Self::Output {
        match self.tiles.get_index2d(index.0) {
            Some(tile) => tile,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T> IndexMut<GridIndex> for TileMap<T> {
    fn index_mut(&mut self, index: GridIndex) -> &mut Self::Output {
        match self.tiles.get_mut_index2d(index.0) {
            Some(tile) => tile,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T: Clone> TileMap<T> {

    pub fn new_with_size_capacity_reference(
        capacity_width: usize,
        capacity_height: usize,
        grid_column_count: usize,
        grid_row_count: usize,
        ref_item: &T
    ) -> Option<Self> {

        let items: Vec<T> = vec![ref_item.clone(); grid_column_count * grid_row_count];
        let tile: Vec2D<T> = Vec2D::new_items_rows_columns(items, grid_row_count, grid_column_count)?;
        let tile_list: Vec<Vec2D<T>> = vec![tile; capacity_width * capacity_height];
        let tiles: Vec2D<Vec2D<T>> = Vec2D::new_items_rows_columns(tile_list, capacity_height, capacity_width)?;

        let grid: MemGrid2D = MemGrid2D::new(tiles.size, grid_row_count, grid_column_count);

        Some(TileMap {
            tiles,
            grid
        })
    }
}


impl<T> TileMap<T> {
    pub fn new(tiles: Vec2D<Vec2D<T>>, tile_rows: usize, tile_columns: usize) -> Self {
        let size: Size2D = tiles.size();

        TileMap {
            tiles,
            grid: MemGrid2D::new(size, tile_rows, tile_columns)
        }
    }

    pub fn get_slice(&self, extents: MemSpan2D) -> Option<TileMapSlice<'_, T>> {
        let (range, intersections) = self.grid.grid_intersections(&extents)?;
        let mut tile_slices: Vec<Vec2DSlice<'_, T>> = Vec::with_capacity(intersections.len());

        for intersection in intersections {

            let cur_slice = self.tiles[intersection.grid_index.0].get_slice(intersection.intersection)?;
            tile_slices.push(cur_slice);
        }

        let tile_slices = Vec2D::new_items_size( tile_slices, range.0.size())?;
        // Some(TileMapSlice
        // {
        //     tile_slices,
        //     row_offsets: Vec::new(),
        //     column_offsets: Vec::new(),
        //     cell_extents: range.0
        // })
        None
    }

}

//TODO: make a range tree to easily access sorted range data.

pub struct TileMapSlice<'a, T>
{
    tile_slices: Vec2D<Vec2DSlice<'a, T>>, //2d grid of row slices
    grid: NonUniformMemGrid2D
}

impl<'a, T> TileMapSlice<'a, T>
{
    #[inline]
    pub fn start_grid_row_count(&self) -> usize {
        self.tile_slices[0].row_count()
    }

    #[inline]
    pub fn start_grid_column_count(&self) -> usize {
        self.tile_slices[0].column_count()
    }

    #[inline]
    pub fn end_grid_row_count(&self) -> usize {
        self.tile_slices[self.tile_slices.row_count() - 1].row_count()
    }

    #[inline]
    pub fn end_grid_column_count(&self) -> usize {
        self.tile_slices[self.tile_slices.column_count() - 1].column_count()
    }

    pub fn get(&self, cell_index2d: MemIndex2D) -> Option<&'a T> {
        let grid_index: GridIndex = self.grid.index2d_to_grid_index(&cell_index2d)?;
        let index_in_grid: MemIndex2D = self.grid.index2d_relative_to_grid(&cell_index2d, &grid_index)?;
        self.tile_slices[grid_index.0].get_index2d(index_in_grid)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_intersections_all() {
        let tiles_rows: usize = 10;
        let tiles_columns: usize = 10;
        let tile: Vec2D<i32> =
            Vec2D::new_size_reference(Size2D::new(tiles_rows, tiles_columns), &0);
        let store: Vec2D<Vec2D<i32>> = Vec2D::new_size_reference(Size2D::new(3, 3), &tile);
        let tiles: TileMap<i32> = TileMap::new(store, tiles_rows, tiles_columns);

        let test_extents: MemSpan2D = MemSpan2D::new_row_columns(30, 30);
        // let (tile_range, intersections) = tiles.all_tile_intersections(test_extents).unwrap(); //should be some
        //
        // assert_eq!(9, intersections.len());
        // assert!(intersections.iter().all(|tile_intersection| tile_intersection.intersection.column_count() == 10 && tile_intersection.intersection.row_count() == 10));
    }

    #[test]
    fn test_tile_intersections_single() {
        let tile_extents: MemSpan2D = MemSpan2D::new_row_columns(10, 10);
        let map_extents: MemSpan2D = MemSpan2D::new_row_columns(3, 3);
        let tile: Vec2D<i32> = Vec2D::new_size_reference(tile_extents.size(), &0);

        let store: Vec2D<Vec2D<i32>> = Vec2D::new_size_reference(map_extents.size(), &tile);
        let tiles: TileMap<i32> = TileMap::new(store, tile.row_count(), tile.column_count());

        let test_extents: MemSpan2D = MemSpan2D::new_row_columns(10, 10);

        let expected_intersection: TileIntersection =
            TileIntersection::new(GridIndex(MemIndex2D::origin()), tile_extents);
        //
        // let (tile_range, intersects) = tiles.all_tile_intersections(test_extents).unwrap(); //should be some
        // assert_eq!(1, intersects.len());
        //
        // assert_eq!(expected_intersection, intersects[0]);
    }

    #[test]
    fn test_tile_intersections_partial_overlap() {
        let tile_extents: MemSpan2D = MemSpan2D::new_row_columns(10, 10);
        let map_extents: MemSpan2D = MemSpan2D::new_row_columns(3, 3);
        let tile: Vec2D<i32> = Vec2D::new_size_reference(tile_extents.size(), &0);
        let store: Vec2D<Vec2D<i32>> = Vec2D::new_size_reference(map_extents.size(), &tile);
        let tiles: TileMap<i32> = TileMap::new(store, tile_extents.row_count(), tile_extents.column_count());

        let test_extents: MemSpan2D = MemSpan2D::new_row_columns(11, 10);

        let expected_intersections: [TileIntersection; 2] = [
            TileIntersection::new(GridIndex(MemIndex2D::origin()), tile_extents),
            TileIntersection::new(GridIndex(MemIndex2D::new(1, 0)), MemSpan2D::new_row_columns(1, 10)),
        ];
        //
        // let intersections = tiles.all_tile_intersections(test_extents).unwrap().1;
        //
        // for (index, intersection) in intersections
        //     .into_iter()
        //     .enumerate()
        // {
        //     println!("{}", intersection);
        //     assert_eq!(expected_intersections[index], intersection);
        // }
    }
}
