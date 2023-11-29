use std::ops::{Index, IndexMut};

use crate::maze::{
    algorithms::dijkstra,
    interface::{Maze, Solution},
};

#[derive(Clone, Debug)]
pub struct RingCell {
    pub coordinates: RingNode,
    pub inaccessible_neighbours: Vec<RingNode>,
    pub accessible_neighbours: Vec<RingNode>,
}

impl RingCell {
    fn carve(&mut self, neighbour: RingNode) {
        if let Some(index) = self
            .inaccessible_neighbours
            .iter()
            .position(|value| *value == neighbour)
        {
            self.inaccessible_neighbours.swap_remove(index);
            self.accessible_neighbours.push(neighbour);
        }
    }

    fn get_walls(&self) -> Vec<RingNode> {
        self.inaccessible_neighbours.clone()
    }

    fn get_paths(&self) -> Vec<RingNode> {
        self.accessible_neighbours.clone()
    }

    fn new(rings: &[usize], coordinates: RingNode) -> Self {
        let mut neighbours: Vec<RingNode> = vec![];
        let ring_max = rings[coordinates.row];

        neighbours.push(RingNode {
            column: if coordinates.column == 0 {
                ring_max - 1
            } else {
                coordinates.column - 1
            },
            ..coordinates
        });

        neighbours.push(RingNode {
            column: if coordinates.column + 1 >= ring_max {
                0
            } else {
                coordinates.column + 1
            },
            ..coordinates
        });

        neighbours.push(RingNode {
            column: if coordinates.row - 1 == 0 {
                0
            } else if rings[coordinates.row - 1] < ring_max {
                coordinates.column / 2
            } else {
                coordinates.column
            },
            row: coordinates.row - 1,
        });

        if coordinates.row + 1 >= rings.len() {
        } else if rings[coordinates.row + 1] > ring_max {
            neighbours.push(RingNode {
                column: coordinates.column * 2,
                row: coordinates.row + 1,
            });
            neighbours.push(RingNode {
                column: coordinates.column * 2 + 1,
                row: coordinates.row + 1,
            });
        } else {
            neighbours.push(RingNode {
                column: coordinates.column,
                row: coordinates.row + 1,
            });
        }

        Self {
            coordinates,
            accessible_neighbours: vec![],
            inaccessible_neighbours: neighbours,
        }
    }
}

impl Index<RingNode> for RingMaze {
    type Output = RingCell;

    fn index(&self, index: RingNode) -> &Self::Output {
        &self.cells[self.get_index(index)]
    }
}

impl IndexMut<RingNode> for RingMaze {
    fn index_mut(&mut self, index: RingNode) -> &mut Self::Output {
        let i = self.get_index(index);
        &mut self.cells[i]
    }
}

pub struct RingMaze {
    pub ring_sizes: Vec<usize>,
    pub cells: Vec<RingCell>,
    extents: Vec<usize>,
}

impl Maze for RingMaze {
    type Idx = RingNode;
    fn carve(&mut self, node: RingNode, neighbour: RingNode) {
        self[node].carve(neighbour);
        self[neighbour].carve(node);
    }

    fn get_walls(&self, node: RingNode) -> Vec<RingNode> {
        self[node].get_walls()
    }

    fn get_paths(&self, node: RingNode) -> Vec<RingNode> {
        self[node].get_paths()
    }

    fn get_random_node(&self) -> RingNode {
        self.cells[fastrand::usize(0..self.cells.len())].coordinates
    }

    fn get_all_edges(&self) -> Vec<(RingNode, RingNode)> {
        let mut frontier = vec![&self.cells[0]];
        let mut edges: Vec<(RingNode, RingNode)> = vec![];
        while let Some(node) = frontier.pop() {
            let new_edges = node
                .inaccessible_neighbours
                .iter()
                .cloned()
                .filter(|n| {
                    n.is_north_of(node.coordinates)
                        || n.is_east_of(node.coordinates, &self.ring_sizes)
                })
                .inspect(|n| {
                    if n.is_north_of(node.coordinates) {
                        frontier.push(&self[*n]);
                    }
                })
                .map(|n| (node.coordinates, n));
            edges.extend(new_edges);
        }

        edges
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        self.cells.iter().map(|c| c.coordinates).collect()
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        node.column + self.extents[node.row]
    }

    fn make_solution(&mut self) -> Solution<RingNode> {
        let start = self.get_random_cell_on_the_outside();
        let exit = self.get_node_furthest_away_from(start);
        let entrance = self.get_node_furthest_away_from(exit);
        let (path_to_solution, distances) = self.find_shortest_path(entrance, exit);
        self.open(entrance);
        self.open(exit);

        Solution {
            path: path_to_solution,
            distances,
        }
    }
}

impl RingMaze {
    /// Ring mazes quickly gain a lot of cells. Since we need to subdivide
    /// the cells per ring for aesthetic reasons, the number of cells grows
    /// 2 ^ (log n) where n is the number of rings.
    /// An 8 * 10 grid has 297 cells (a rectilinear grid would just have 80).
    fn compute_no_of_columns(row: usize, column_factor: usize) -> usize {
        2_usize.pow(row.ilog2()) * column_factor
    }

    /// Cells are stored in a flat vector. The index implementation for `RingNode`
    /// finds out how many cells are in each ring via the ring sizes.
    /// Vector index of (r, c) = sum of ring sizes up to r + c
    fn compute_cells(max_rings: usize, rings: &[usize], column_factor: usize) -> Vec<RingCell> {
        let mut cells = vec![RingCell {
            coordinates: RingNode { row: 0, column: 0 },
            inaccessible_neighbours: (0..column_factor)
                .map(|column| RingNode { row: 1, column })
                .collect(),
            accessible_neighbours: vec![],
        }];

        cells.extend((1..max_rings).flat_map(|row| {
            (0..Self::compute_no_of_columns(row, column_factor))
                .map(move |column| RingCell::new(rings, RingNode { row, column }))
        }));

        cells
    }

    pub fn new(max_rings: usize, column_factor: usize) -> RingMaze {
        let mut rings = vec![1];
        rings.extend((1..max_rings).map(|row| Self::compute_no_of_columns(row, column_factor)));
        let extents = (0..max_rings)
            .map(|x| rings[0..x].iter().sum::<usize>())
            .collect::<Vec<_>>();

        let cells = Self::compute_cells(max_rings, &rings, column_factor);
        RingMaze {
            ring_sizes: rings,
            extents,
            cells,
        }
    }

    /// No bounds checking on `ring`. Panics if `ring` ≥ `ring_sizes.len()` of this maze
    pub fn max_column(&self, ring: usize) -> usize {
        self.ring_sizes[ring]
    }

    fn get_random_cell_on_the_outside(&self) -> RingNode {
        let ring = self.ring_sizes.len() - 1;
        let column = fastrand::usize(0..self.ring_sizes[ring]);
        RingNode { row: ring, column }
    }

    fn find_shortest_path(&self, start: RingNode, end: RingNode) -> (Vec<RingNode>, Vec<usize>) {
        let distances = dijkstra(self, start);
        let mut cursor = end;
        let mut path = vec![cursor];
        loop {
            cursor = *self
                .get_paths(cursor)
                .iter()
                .min_by_key(|n| distances[self.get_index(**n)])
                .unwrap();
            path.push(cursor);
            if distances[self.get_index(cursor)] == 1 {
                break;
            }
        }
        path.push(start);

        (path, distances)
    }

    fn open(&mut self, node: RingNode) {
        self[node].accessible_neighbours.push(RingNode {
            row: node.row + 1,
            column: 0,
        })
    }

    fn get_node_furthest_away_from(&self, start: RingNode) -> RingNode {
        let outer_ring = self.ring_sizes.len() - 1;
        let topo = dijkstra(self, start);
        RingNode {
            row: outer_ring,
            column: (0..self.ring_sizes[outer_ring])
                .max_by_key(|column| {
                    topo[self.get_index(RingNode {
                        column: *column,
                        row: outer_ring,
                    })]
                })
                .unwrap(),
        }
    }
}

// we only need hash for one test. it's not used in the runtime code
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RingNode {
    pub row: usize,
    pub column: usize,
}

impl RingNode {
    pub fn is_north_of(&self, other: RingNode) -> bool {
        self.row > other.row
    }

    pub fn is_south_of(&self, other: RingNode) -> bool {
        self.row < other.row
    }

    pub fn is_east_of(&self, other: RingNode, extents: &[usize]) -> bool {
        self.row == other.row && (self.column + 1) % extents[self.row] == other.column
    }

    pub fn is_west_of(&self, other: RingNode, extents: &[usize]) -> bool {
        self.row == other.row && self.column == (other.column + 1 % extents[self.row])
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn east_west_of_wraps_around_properly() {
        let rings = RingMaze::new(6, 8).ring_sizes;
        let node = RingNode { row: 1, column: 0 };
        assert!(RingNode { row: 1, column: 1 }.is_west_of(node, &rings));
        assert!(
            !RingNode { row: 1, column: 7 }.is_west_of(node, &rings),
            "Should not be directly west of a node that wrapped around the eastern end"
        );

        assert!(!RingNode { row: 1, column: 1 }.is_east_of(node, &rings));
        assert!(RingNode { row: 1, column: 7 }.is_east_of(node, &rings));
    }

    #[test]
    fn edge_count_is_correct() {
        let maze = RingMaze::new(5, 8);
        let expected_edge_count = 2 * 8 + 2 * 16 + 2 * 16 + 2 * 32;
        assert_eq!(maze.get_all_edges().len(), expected_edge_count);
    }

    #[test]
    fn no_edge_contains_itself_in_reverse() {
        let maze = RingMaze::new(5, 8);
        let edges = maze.get_all_edges();
        for (a, b) in edges.iter() {
            assert!(
                !edges.contains(&(*b, *a)),
                "Edge ({}, {})-({}, {}) and its reciprocal shouldn't both be in the edge list",
                a.row,
                a.column,
                b.row,
                b.column
            );
        }
    }

    #[test]
    fn no_edge_should_occur_twice() {
        let maze = RingMaze::new(5, 8);
        let edges = maze.get_all_edges();
        let unique_edges = edges.iter().unique().collect::<Vec<_>>();
        assert_eq!(
            edges.len(),
            unique_edges.len(),
            "Edge count of unique edges and total edges should be the same"
        );
    }

    #[test]
    fn indices_are_computed_correctly() {
        let maze = RingMaze::new(5, 8);
        assert_eq!(
            maze.get_index(RingNode { row: 0, column: 0 }),
            0,
            "Index of (0, 0) should be 0"
        );
        assert_eq!(
            maze.get_index(RingNode { row: 1, column: 0 }),
            1,
            "Index of (1, 0) should be 1"
        );
        assert_eq!(
            maze.get_index(RingNode { row: 2, column: 0 }),
            1 + 8,
            "Index of (2, 0) should be 1 + 8"
        );
        assert_eq!(
            maze.get_index(RingNode { row: 2, column: 7 }),
            1 + 8 + 7,
            "Index of (2, 1) should be 1 + 8 + 7"
        );
        assert_eq!(
            maze.get_index(RingNode { row: 3, column: 7 }),
            1 + 8 + 16 + 7,
            "Index of (2, 1) should be 1 + 8 + 16 + 7"
        )
    }
}
