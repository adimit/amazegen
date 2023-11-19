use std::ops::{Index, IndexMut};

use svg::node::element::path::Command::{self, EllipticalArc};
use svg::node::element::path::Data;
use svg::node::element::path::Position::Absolute;
use svg::node::element::Path;
use svg::{Document, Node};

const π: f64 = std::f64::consts::PI;

// Rough sketch:
// Each ring is a vector of polar cells.
// Each polar cell has a set of polar coordinates and a list of inaccessible neighbours,
// which are polar coordinates, and a list of accessible neigbours (initially empty).
// You can use polar coordinates to access the cell structure.
// To render walls of a cell, you ask it whether any of its inaccessible neighbours has coordinates
// with a greater row or column than itself. If yes, you draw the wall.
// This elegantly handles the case of split cells in higher rings: if any of the cells neighbours
// higher up is accessible, we just don't draw the northern wall at all. The outer cells get to
// draw their southern walls instead.
// The tricky bit is populating the inaccessible neighbours list. East & West are easy. South
// can't rely on this cell's own index, as the southerly neigbour's index may be half of its own.
// Similarly, the northern neighbours's index may be twice its own, and another one + 1. So it needs
// to be aware of the `compute_no_of_columns` function's implementation.

#[derive(Clone, Debug)]
struct RingCell {
    coordinates: RingNode,
    inaccessible_neighbours: Vec<RingNode>,
    accessible_neighbours: Vec<RingNode>,
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

    fn new(rings: &Vec<usize>, coordinates: RingNode) -> Self {
        let mut neighbours: Vec<RingNode> = vec![];
        let ring_max = rings[coordinates.row];

        neighbours.push(RingNode {
            column: if coordinates.column == 0 {
                ring_max - 1
            } else {
                coordinates.column - 1
            },
            ..coordinates.clone()
        });

        neighbours.push(RingNode {
            column: if coordinates.column + 1 >= ring_max {
                0
            } else {
                coordinates.column + 1
            },
            ..coordinates.clone()
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

#[cfg(test)]
mod test {
    use super::*;
}

impl Index<RingNode> for RingMaze {
    type Output = RingCell;

    fn index(&self, index: RingNode) -> &Self::Output {
        &self.cells[index.column + self.ring_sizes[0..index.row].iter().sum::<usize>()]
    }
}

impl IndexMut<RingNode> for RingMaze {
    fn index_mut(&mut self, index: RingNode) -> &mut Self::Output {
        &mut self.cells[index.column + self.ring_sizes[0..index.row].iter().sum::<usize>()]
    }
}

struct RingMaze {
    ring_sizes: Vec<usize>,
    cells: Vec<RingCell>,
}

const COLUMN_FACTOR: usize = 8;

impl RingMaze {
    /// Ring mazes quickly gain a lot of cells. Since we need to subdivide
    /// the cells per ring for aesthetic reasons, the number of cells grows
    /// 2 ^ (log n) where n is the number of rings.
    /// An 8 * 10 grid has 297 cells (a rectilinear grid would just have 80).
    fn compute_no_of_columns(row: usize, column_factor: usize) -> usize {
        2_usize.pow(row.ilog2()) * column_factor
    }

    /// Cells are stored in a flat vector. The index implementation for `RindNode`
    /// finds out how many cells are in each ring via the ring sizes.
    /// Vector index of (r, c) = sum of ring sizes up to r + c
    fn compute_cells(max_rings: usize, rings: &Vec<usize>, column_factor: usize) -> Vec<RingCell> {
        let mut cells = vec![RingCell {
            coordinates: RingNode { row: 0, column: 0 },
            inaccessible_neighbours: (0..COLUMN_FACTOR)
                .map(|column| RingNode { row: 0, column })
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

        let cells = Self::compute_cells(max_rings, &rings, column_factor);
        RingMaze {
            ring_sizes: rings,
            cells,
        }
    }

    pub fn carve(&mut self, node: RingNode, neighbour: RingNode) {
        self[node].carve(neighbour);
        self[neighbour].carve(node);
    }

    pub fn get_walls(&self, node: RingNode) -> Vec<RingNode> {
        self[node].get_walls()
    }

    pub fn get_paths(&self, node: RingNode) -> Vec<RingNode> {
        self[node].get_paths()
    }

    pub fn get_random_node(&self) -> RingNode {
        self.cells[fastrand::usize(0..self.cells.len())].coordinates
    }

    pub fn get_all_edges(&self) -> Vec<(RingNode, RingNode)> {
        todo!()
    }

    /// No bounds checking on `ring`. Panics if `ring` ≥ `ring_sizes.len()` of this maze
    fn max_column(&self, ring: usize) -> usize {
        self.ring_sizes[ring]
    }
}

fn jarník(mut maze: RingMaze) -> RingMaze {
    let mut vertices: Vec<RingNode> = vec![maze.get_random_node()];
    fastrand::seed(7);

    while !vertices.is_empty() {
        let i = vertices.len() - 1;
        let e = vertices[i];
        let targets = maze.get_walls(e);
        if !targets.is_empty() {
            let target = targets[fastrand::usize(0..targets.len())];
            maze.carve(e, target);
            vertices.push(target);
        } else {
            vertices.swap_remove(i);
        }
    }

    maze
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RingNode {
    pub row: usize,
    pub column: usize,
}

struct PolarGrid<'a> {
    ring_height: f64,
    maze: &'a RingMaze,
}

impl PolarGrid<'_> {
    fn new(maze: &RingMaze, ring_height: f64) -> PolarGrid {
        PolarGrid { maze, ring_height }
    }

    fn θ(&self, row: usize) -> f64 {
        // TODO: cache theta values?
        2.0 * π / self.maze.max_column(row) as f64
    }

    fn inner_radius(&self, row: usize) -> f64 {
        self.ring_height * row as f64
    }

    fn outer_radius(&self, row: usize) -> f64 {
        self.ring_height * (row + 1) as f64
    }

    fn θ_west(&self, node: RingNode) -> f64 {
        self.θ(node.row) * (1.0 + node.column as f64)
    }

    fn θ_east(&self, node: RingNode) -> f64 {
        self.θ(node.row) * (node.column as f64)
    }

    fn compute_cell(&self, node: RingNode) -> CellCoordinates {
        let inner = self.inner_radius(node.row);
        let outer = self.outer_radius(node.row);
        let east = self.θ_east(node);
        let west = self.θ_west(node);
        let centre = CartesianPoint { x: 500, y: 500 };

        CellCoordinates {
            ax: centre.x as f64 + (inner * west.cos()),
            ay: centre.y as f64 + (inner * west.sin()),
            bx: centre.x as f64 + (outer * west.cos()),
            by: centre.y as f64 + (outer * west.sin()),
            cx: centre.x as f64 + (inner * east.cos()),
            cy: centre.y as f64 + (inner * east.sin()),
            dx: centre.x as f64 + (outer * east.cos()),
            dy: centre.y as f64 + (outer * east.sin()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CartesianPoint {
    x: u64,
    y: u64,
}

#[derive(Copy, Clone, Debug)]
struct CellCoordinates {
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
}

fn make_maze() -> RingMaze {
    let mut fresh = RingMaze::new(12, 9);
    jarník(fresh)
}

pub fn test_maze() -> Result<(), ()> {
    println!("Generating maze...");
    let maze = make_maze();
    let grid = PolarGrid::new(&maze, 40.0);
    let mut document = Document::new().set("viewBox", (0, 0, 1000, 1000));

    fn arc(data: &mut Data, grid: &PolarGrid, column: usize, row: usize) {
        let node = RingNode { column, row };
        let cell = grid.compute_cell(node);
        let outer = grid.outer_radius(node.row);
        let inner = grid.inner_radius(node.row);
        data.append(Command::Move(Absolute, (cell.ax, cell.ay).into()));
        data.append(Command::Line(Absolute, (cell.bx, cell.by).into()));
        data.append(EllipticalArc(
            Absolute,
            (outer, outer, 0, 0, 0, cell.dx, cell.dy).into(),
        ));
        data.append(Command::Move(Absolute, (cell.cx, cell.cy).into()));
        data.append(EllipticalArc(
            Absolute,
            (inner, inner, 0, 1, 0, cell.ax, cell.ay).into(),
        ));
    }

    let mut data = Data::new();
    for row in 1..maze.ring_sizes.len() {
        for column in 0..maze.max_column(row) {
            arc(&mut data, &grid, column, row)
        }
    }
    let path = Path::new()
        .set("stroke", "black")
        .set("fill", "none")
        .set("d", data)
        .set("stroke-width", "3");
    document.append(path);

    svg::save("test-output.svg", &document).unwrap();

    Ok(())
}
