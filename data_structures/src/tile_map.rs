use std::{
    clone, fmt,
    fmt::Pointer,
    ops::{Index, IndexMut},
};

use memory_math::{
    memory_extents2d::{HasMemExtents2D, MemExtents2D},
    memory_index2d::MemIndex2D,
    memory_vect2d::MemVect2D,
};

use super::{
    iter_index2d::CanIterIndex2D, vec2d::Vec2D, vec2d_iter::Vec2DIntoIter, vec_2d::Vector2D,
};

//A TileMap is a 2d Vec containing uniformy sized, compact, square tiles
// of size nxn. This way you can perform operations like get_tile_at(2,2) \
//which will return a reference to the tile which contains the coordinate (2,2)
#[derive(Clone)]
pub struct TileMap<T: HasMemExtents2D> {
    pub tile_extents: MemExtents2D,
    tiles: Vec2D<T>,
}

pub struct TileIntersection {
    pub tile_index: MemIndex2D,
    pub intersection: MemExtents2D,
}

impl PartialEq for TileIntersection {
    fn eq(&self, other: &Self) -> bool {
        self.tile_index == other.tile_index && self.intersection == other.intersection
    }
}

impl TileIntersection {
    fn new(tile_index: MemIndex2D, intersection: MemExtents2D) -> Self {
        TileIntersection {
            tile_index,
            intersection,
        }
    }
}

impl fmt::Display for TileIntersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TileIntersection[tile_index: {}, intersection: {}]",
            self.tile_index, self.intersection
        )
    }
}

impl fmt::Debug for TileIntersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileIntersection")
            .field("tile_index", &self.tile_index)
            .field("intersection", &self.intersection)
            .finish()
    }
}

pub enum ExtentsScope {
    TileExtents(MemExtents2D),
    CellExtents(MemExtents2D),
}

pub enum CoordinateType {
    //A coordinate pointing at a specific tile
    TileIndex(MemIndex2D),
    //A coordinate pointing to a global cell located inside the span of all tiles
    CellIndex(MemIndex2D),
}

impl<T: HasMemExtents2D> IntoIterator for TileMap<T> {
    type Item = T;

    type IntoIter = Vec2DIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.into_iter()
    }
}

impl<T: HasMemExtents2D> IndexMut<CoordinateType> for TileMap<T> {
    fn index_mut(&mut self, index: CoordinateType) -> &mut Self::Output {
        let tile_index: MemIndex2D = self.coordinate_to_tile_index2d(index);

        match self.tiles.get_mut(tile_index) {
            Some(tile) => tile,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T: HasMemExtents2D> HasMemExtents2D for TileMap<T> {
    fn get_extents(&self) -> Result<MemExtents2D, &'static str> {
        self.tiles.get_extents()
    }
}

impl<T: HasMemExtents2D> CanIterIndex2D for TileMap<T> {}

impl<T: HasMemExtents2D> Index<CoordinateType> for TileMap<T> {
    type Output = T;

    fn index(&self, index: CoordinateType) -> &Self::Output {
        let tile_index: MemIndex2D = self.coordinate_to_tile_index2d(index);

        match self.tiles.get_ref(tile_index) {
            Some(tile) => tile,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<T: HasMemExtents2D + Clone> TileMap<T> {
    pub fn new_with_size_capacity_reference(
        tile_extents: MemExtents2D,
        capacity_width: usize,
        capacity_height: usize,
        ref_item: &T,
    ) -> Self {
        let tiles: Vec2D<T> = Vec2D::new_size_reference(capacity_width, capacity_height, ref_item);

        TileMap {
            tile_extents,
            tiles,
        }
    }
}

impl<T: HasMemExtents2D> TileMap<T> {
    pub fn new(tiles: Vec2D<T>, extents: MemExtents2D) -> Self {
        TileMap {
            tile_extents: extents,
            tiles,
        }
    }

    pub fn in_range(&self, coordinate: CoordinateType) -> bool {
        match coordinate {
            CoordinateType::TileIndex(tile) => return self.tiles.index2d_in_bounds(tile),
            CoordinateType::CellIndex(cell) => {
                return cell.row <= self.get_cell_height_span()
                    && cell.col <= self.get_cell_width_span()
            }
        }
    }

    pub fn get_cell_height_span(&self) -> usize {
        self.tiles.height() * self.tile_extents.height()
    }

    pub fn get_cell_width_span(&self) -> usize {
        self.tiles.width() * self.tile_extents.width()
    }

    pub fn cell_coordinate_to_tile_index(&self, cell_coordinate: MemIndex2D) -> MemIndex2D {
        MemIndex2D::new(
            cell_coordinate.row / self.tile_extents.height(),
            cell_coordinate.col / self.tile_extents.width(),
        )
    }

    pub fn coordinate_to_index_in_tile(
        &self,
        index: CoordinateType,
        tile_index: MemIndex2D,
    ) -> Option<MemIndex2D> {
        match index {
            CoordinateType::TileIndex(tile_coord) => Some(MemIndex2D::origin()),
            CoordinateType::CellIndex(cell_coord) => {
                cell_coord - MemVect2D::from(self.tile_index_to_cell_index(tile_index))
            }
        }
    }

    pub fn tile_index_to_cell_index(&self, tile_index: MemIndex2D) -> MemIndex2D {
        MemIndex2D::new(
            tile_index.row * self.tile_extents.height(),
            tile_index.col * self.tile_extents.width(),
        )
    }

    pub fn coordinate_to_tile_index2d(&self, coordinate: CoordinateType) -> MemIndex2D {
        match coordinate {
            CoordinateType::CellIndex(index) => self.cell_coordinate_to_tile_index(index),
            CoordinateType::TileIndex(tile_index) => tile_index,
        }
    }

    pub fn coordinate_to_cell_index2d(&self, coordinate: CoordinateType) -> MemIndex2D {
        match coordinate {
            CoordinateType::CellIndex(coord) => coord,
            CoordinateType::TileIndex(tile_coord) => MemIndex2D::new(
                tile_coord.row * self.tile_extents.height(),
                tile_coord.col * self.tile_extents.width(),
            ),
        }
    }

    pub fn try_intersect_tile(
        &self,
        tile_index: MemIndex2D,
        range: MemExtents2D,
    ) -> Option<MemExtents2D> {
        let shift_vect =
            MemVect2D::from(self.coordinate_to_cell_index2d(CoordinateType::TileIndex(tile_index)));
        let normalized_tile_extents = self.tile_extents + shift_vect;

        let shifted_intersection = match range.intersect(normalized_tile_extents) {
            Some(overlap) => overlap,
            None => {
                return None;
            }
        };

        if shifted_intersection.get_min_coord().row == shifted_intersection.get_max_coord().row
            || shifted_intersection.get_min_coord().col == shifted_intersection.get_max_coord().col
        {
            return None;
        }

        shifted_intersection - shift_vect
    }

    pub fn get_tile_intersections(&self, extents: MemExtents2D) -> Vec<TileIntersection> {
        let mut intersections: Vec<TileIntersection> = Vec::with_capacity(self.tiles.len());

        let iter = match self.iter_index2d() {
            Ok(val) => val,
            Err(err_str) => panic!("Cannot get tile intersections: {}", err_str),
        };

        for index in iter {
            let tile_intersection = match self.try_intersect_tile(index, extents) {
                Some(val) => val,
                None => {
                    continue;
                }
            };

            intersections.push(TileIntersection {
                tile_index: index,
                intersection: tile_intersection,
            });
        }

        intersections
    }

    pub fn extents_to_tile_extents(
        &self,
        extents: ExtentsScope,
    ) -> Result<MemExtents2D, &'static str> {
        match extents {
            ExtentsScope::TileExtents(ext) => Ok(ext),
            ExtentsScope::CellExtents(ext) => MemExtents2D::new_from_coords(
                self.cell_coordinate_to_tile_index(ext.get_min_coord()),
                self.cell_coordinate_to_tile_index(ext.get_max_coord()),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tile_intersections_all() {
        let tile_extents: MemExtents2D = MemExtents2D::new_width_height(10, 10);
        let map_extents: MemExtents2D = MemExtents2D::new_width_height(3, 3);
        let store: Vec2D<MemExtents2D> =
            Vec2D::new_from_extents_reference(map_extents, &tile_extents);
        let tiles: TileMap<MemExtents2D> = TileMap::new(store, tile_extents);

        let test_extents: MemExtents2D = MemExtents2D::new_width_height(30, 30);
        let expected_intersections: [TileIntersection; 9] = [
            TileIntersection::new(MemIndex2D::origin(), tile_extents),
            TileIntersection::new(MemIndex2D::new(0, 1), tile_extents),
            TileIntersection::new(MemIndex2D::new(0, 2), tile_extents),
            TileIntersection::new(MemIndex2D::new(1, 0), tile_extents),
            TileIntersection::new(MemIndex2D::new(1, 1), tile_extents),
            TileIntersection::new(MemIndex2D::new(1, 2), tile_extents),
            TileIntersection::new(MemIndex2D::new(2, 0), tile_extents),
            TileIntersection::new(MemIndex2D::new(2, 1), tile_extents),
            TileIntersection::new(MemIndex2D::new(2, 2), tile_extents),
        ];

        for (index, intersection) in tiles
            .get_tile_intersections(test_extents)
            .into_iter()
            .enumerate()
        {
            assert_eq!(expected_intersections[index], intersection)
        }
    }

    #[test]
    fn test_tile_intersections_single() {
        let tile_extents: MemExtents2D = MemExtents2D::new_width_height(10, 10);
        let map_extents: MemExtents2D = MemExtents2D::new_width_height(3, 3);
        let store: Vec2D<MemExtents2D> =
            Vec2D::new_from_extents_reference(map_extents, &tile_extents);
        let tiles: TileMap<MemExtents2D> = TileMap::new(store, tile_extents);

        let test_extents: MemExtents2D = MemExtents2D::new_width_height(10, 10);

        let expected_intersection: TileIntersection =
            TileIntersection::new(MemIndex2D::origin(), tile_extents);

        let intersects = tiles.get_tile_intersections(test_extents);
        assert_eq!(1, intersects.len());

        assert_eq!(expected_intersection, intersects[0]);
    }

    #[test]
    fn test_tile_intersections_partial_overlap() {
        let tile_extents: MemExtents2D = MemExtents2D::new_width_height(10, 10);
        let map_extents: MemExtents2D = MemExtents2D::new_width_height(3, 3);
        let store: Vec2D<MemExtents2D> =
            Vec2D::new_from_extents_reference(map_extents, &tile_extents);
        let tiles: TileMap<MemExtents2D> = TileMap::new(store, tile_extents);

        let test_extents: MemExtents2D = MemExtents2D::new_width_height(11, 10);

        let expected_intersections: [TileIntersection; 2] = [
            TileIntersection::new(MemIndex2D::origin(), tile_extents),
            TileIntersection::new(MemIndex2D::new(0, 1), MemExtents2D::new_width_height(1, 10)),
        ];

        for (index, intersection) in tiles
            .get_tile_intersections(test_extents)
            .into_iter()
            .enumerate()
        {
            println!("{}", intersection);
            assert_eq!(expected_intersections[index], intersection);
        }
    }
}
