use std::ops::{Index, IndexMut};

use svg::node::element::path::Command::{self, EllipticalArc};
use svg::node::element::path::Data;
use svg::node::element::path::Position::Absolute;
use svg::node::element::Path;
use svg::{Document, Node};

const π: f64 = std::f64::consts::PI;

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

#[derive(Debug)]
struct Cells<T> {
    ring_sizes: Vec<usize>,
    cells: Vec<T>,
    extents: Vec<usize>,
}

impl<T> Cells<T>
where
    T: Copy + Clone,
{
    pub fn new(ring_sizes: &Vec<usize>, default: T) -> Self {
        let extents = ring_sizes
            .iter()
            .scan(0, |state, &x| {
                *state += x;
                Some(*state)
            })
            .collect::<Vec<_>>();
        Self {
            ring_sizes: ring_sizes.clone(),
            cells: vec![default; ring_sizes.iter().sum::<usize>()],
            extents,
        }
    }
}

impl<T> Index<RingNode> for Cells<T> {
    type Output = T;

    fn index(&self, index: RingNode) -> &Self::Output {
        /*
            assert_eq!(
                self.ring_sizes[0..index.row].iter().sum::<usize>(),
                self.extents[index.row]
        );
            */
        &self.cells[index.column + self.ring_sizes[0..index.row].iter().sum::<usize>()]
    }
}

impl<T> IndexMut<RingNode> for Cells<T> {
    fn index_mut(&mut self, index: RingNode) -> &mut Self::Output {
        /*
            assert_eq!(
                self.ring_sizes[0..index.row].iter().sum::<usize>(),
                self.extents[index.row]
        );
            */
        &mut self.cells[index.column + self.ring_sizes[0..index.row].iter().sum::<usize>()]
    }
}

fn dijkstra(maze: &RingMaze, origin: RingNode) -> Cells<usize> {
    let mut distances = Cells::new(&maze.ring_sizes, 0);
    let mut frontier: Vec<RingNode> = vec![origin];
    while !frontier.is_empty() {
        let mut new_frontier: Vec<RingNode> = vec![];
        for cell in frontier.drain(..) {
            for new in maze.get_paths(cell) {
                if distances[new] == 0 {
                    distances[new] = distances[cell] + 1;
                    new_frontier.push(new);
                }
            }
        }
        frontier.append(&mut new_frontier);
    }
    distances
}

struct RingMaze {
    ring_sizes: Vec<usize>,
    cells: Vec<RingCell>,
}

trait JarníkMaze {
    type Idx: PartialEq + Copy + Clone;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx);

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_random_node(&self) -> Self::Idx;

    fn open(&mut self, node: Self::Idx);

    fn dijkstra(&self, origin: Self::Idx) -> Cells<usize>;
}

impl JarníkMaze for RingMaze {
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

    fn open(&mut self, node: RingNode) {
        self[node].accessible_neighbours.push(RingNode {
            row: node.row + 1,
            column: 0,
        })
    }

    fn dijkstra(&self, origin: RingNode) -> Cells<usize> {
        dijkstra(self, origin)
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

    /// Cells are stored in a flat vector. The index implementation for `RindNode`
    /// finds out how many cells are in each ring via the ring sizes.
    /// Vector index of (r, c) = sum of ring sizes up to r + c
    fn compute_cells(max_rings: usize, rings: &Vec<usize>, column_factor: usize) -> Vec<RingCell> {
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

    fn new(max_rings: usize, column_factor: usize) -> RingMaze {
        let mut rings = vec![1];
        rings.extend((1..max_rings).map(|row| Self::compute_no_of_columns(row, column_factor)));

        let cells = Self::compute_cells(max_rings, &rings, column_factor);
        RingMaze {
            ring_sizes: rings,
            cells,
        }
    }

    fn get_all_edges(&self) -> Vec<(RingNode, RingNode)> {
        todo!()
    }

    /// No bounds checking on `ring`. Panics if `ring` ≥ `ring_sizes.len()` of this maze
    fn max_column(&self, ring: usize) -> usize {
        self.ring_sizes[ring]
    }
}

fn jarník<M: JarníkMaze>(mut maze: M) -> M {
    let start = maze.get_random_node();
    let mut vertices: Vec<M::Idx> = vec![start];
    let mut visited: Vec<M::Idx> = vec![start];

    while !vertices.is_empty() {
        let i = vertices.len() - 1;
        let e = vertices[i];
        let targets = maze
            .get_walls(e)
            .iter()
            .filter(|n| !visited.contains(n))
            .cloned()
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            let target = targets[fastrand::usize(0..targets.len())];
            maze.carve(e, target);
            vertices.push(target);
        } else {
            vertices.swap_remove(i);
        }
        visited.push(e);
    }

    maze
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub fn is_east_of(&self, other: RingNode, extents: &Vec<usize>) -> bool {
        self.row == other.row
            && (self.column + 1 == other.column
                || (other.column == 0 && self.column == extents[self.row] - 1))
    }

    pub fn is_west_of(&self, other: RingNode, extents: &Vec<usize>) -> bool {
        self.row == other.row
            && (self.column == other.column + 1
                || (self.column == 0 && other.column == extents[self.row] - 1))
    }
}

#[cfg(test)]
mod test {
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
}

struct PolarGrid<'a> {
    ring_height: f64,
    maze: &'a RingMaze,
    centre: CartesianPoint,
}

impl PolarGrid<'_> {
    fn new(maze: &RingMaze, ring_height: f64, stroke_width: f64) -> PolarGrid {
        PolarGrid {
            maze,
            ring_height,
            centre: CartesianPoint {
                x: ring_height * maze.ring_sizes.len() as f64 + stroke_width,
                y: ring_height * maze.ring_sizes.len() as f64 + stroke_width,
            },
        }
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

        CellCoordinates {
            ax: self.centre.x as f64 + (inner * west.cos()),
            ay: self.centre.y as f64 + (inner * west.sin()),
            bx: self.centre.x as f64 + (outer * west.cos()),
            by: self.centre.y as f64 + (outer * west.sin()),
            cx: self.centre.x as f64 + (inner * east.cos()),
            cy: self.centre.y as f64 + (inner * east.sin()),
            dx: self.centre.x as f64 + (outer * east.cos()),
            dy: self.centre.y as f64 + (outer * east.sin()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CartesianPoint {
    x: f64,
    y: f64,
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

fn get_node_furthest_away_from(maze: &RingMaze, start: RingNode) -> RingNode {
    let outer_ring = maze.ring_sizes.len() - 1;
    let topo = maze.dijkstra(start);
    RingNode {
        row: outer_ring,
        column: (0..maze.ring_sizes[outer_ring])
            .max_by_key(|column| {
                topo[RingNode {
                    column: *column,
                    row: outer_ring,
                }]
            })
            .unwrap(),
    }
}

fn make_maze(rings: usize, column_factor: usize) -> (RingMaze, Vec<RingNode>) {
    //fastrand::seed(10);
    let mut maze = jarník(RingMaze::new(rings, column_factor));
    let start = get_random_cell_on_the_outside(&maze);
    let exit = get_node_furthest_away_from(&maze, start);
    let entrance = get_node_furthest_away_from(&maze, exit);
    let path_to_solution = find_shortest_path(&maze, entrance, exit);
    maze.open(entrance);
    maze.open(exit);

    (maze, path_to_solution)
}

fn debug_maze(maze: &RingMaze) {
    for cell in maze.cells.iter() {
        print!("({}, {}): ", cell.coordinates.row, cell.coordinates.column);
        for neighbour in cell.inaccessible_neighbours.iter() {
            print!("[{}, {}] ", neighbour.row, neighbour.column);
        }
        print!("| ");
        for neighbour in cell.accessible_neighbours.iter() {
            print!("({}, {}) ", neighbour.row, neighbour.column);
        }
        println!();
    }
}

fn find_shortest_path(maze: &RingMaze, start: RingNode, end: RingNode) -> Vec<RingNode> {
    let distances = maze.dijkstra(start);
    let mut cursor = end;
    let mut path = vec![cursor];
    loop {
        cursor = *maze
            .get_paths(cursor)
            .iter()
            .min_by_key(|n| distances[**n])
            .unwrap();
        path.push(cursor);
        if distances[cursor] == 1 {
            break;
        }
    }
    path.push(start);

    path
}

fn middle(grid: &PolarGrid, node: &RingNode) -> (f64, f64) {
    if node.row == 0 && node.column == 0 {
        return (grid.centre.x as f64, grid.centre.y as f64);
    }
    let r = grid.inner_radius(node.row) + grid.ring_height / 2.0;
    let θ = grid.θ_east(*node) + grid.θ(node.row) / 2.0;
    (
        grid.centre.x as f64 + (r * θ.cos()),
        grid.centre.y as f64 + (r * θ.sin()),
    )
}

fn draw_path(grid: &PolarGrid, path: &Vec<RingNode>) -> Path {
    let mut data = Data::new();
    data.append(Command::Move(Absolute, middle(grid, &path[0]).into()));
    for node in path.iter().skip(1) {
        data.append(Command::Line(Absolute, middle(grid, node).into()));
    }
    Path::new()
        .set("stroke", "red")
        .set("fill", "none")
        .set("d", data)
        .set("stroke-width", "3")
}

fn get_random_cell_on_the_outside(maze: &RingMaze) -> RingNode {
    let ring = maze.ring_sizes.len() - 1;
    let column = fastrand::usize(0..maze.ring_sizes[ring]);
    RingNode { row: ring, column }
}

pub fn test_maze() -> Result<(), ()> {
    let height = 40.0;
    let rings = 12;
    let column_factor = 8;
    let (maze, path_to_solution) = make_maze(rings, column_factor);
    let stroke_width = 3.0;
    let grid = PolarGrid::new(&maze, height, stroke_width);
    let pixels = (grid.centre.x + stroke_width) * 2.0;
    let mut document = Document::new().set("viewBox", (0, 0, pixels, pixels));

    fn render_cell(data: &mut Data, grid: &PolarGrid, cell: &RingCell) {
        let node = cell.coordinates;
        let c = grid.compute_cell(node);
        let outer = grid.outer_radius(node.row);
        let inner = grid.inner_radius(node.row);

        // east wall
        if cell
            .inaccessible_neighbours
            .iter()
            .any(|it| it.is_east_of(node, &grid.maze.ring_sizes))
        {
            data.append(Command::Move(Absolute, (c.cx, c.cy).into()));
            data.append(Command::Line(Absolute, (c.dx, c.dy).into()));
        }

        // west wall
        if cell
            .inaccessible_neighbours
            .iter()
            .any(|it| it.is_west_of(node, &grid.maze.ring_sizes))
        {
            data.append(Command::Move(Absolute, (c.ax, c.ay).into()));
            data.append(Command::Line(Absolute, (c.bx, c.by).into()));
        }

        // north wall
        if !cell
            .accessible_neighbours
            .iter()
            .any(|it| it.is_north_of(node))
        {
            data.append(Command::Move(Absolute, (c.bx, c.by).into()));
            data.append(EllipticalArc(
                Absolute,
                (outer, outer, 0, 0, 0, c.dx, c.dy).into(),
            ));
        }

        // south wall
        if cell
            .inaccessible_neighbours
            .iter()
            .any(|it| it.is_south_of(node))
        {
            data.append(Command::Move(Absolute, (c.cx, c.cy).into()));
            data.append(EllipticalArc(
                Absolute,
                (inner, inner, 0, 0, 1, c.ax, c.ay).into(),
            ));
        }
    }

    let mut data = Data::new();
    for node in maze.cells.iter() {
        render_cell(&mut data, &grid, node);
    }

    let path = Path::new()
        .set("stroke", "black")
        .set("fill", "none")
        .set("d", data)
        .set("stroke-width", "3");
    document.append(path);

    // let solution = draw_path(&grid, &path_to_solution);
    // document.append(solution);

    svg::save("test-output.svg", &document).unwrap();

    Ok(())
}
