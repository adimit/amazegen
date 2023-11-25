use std::cmp::min;
use std::ops::{Index, IndexMut};

use svg::node::element::path::Command::{self, EllipticalArc};
use svg::node::element::path::Position::Absolute;
use svg::node::element::path::{Data, Parameters};
use svg::node::element::{Circle, Path};
use svg::{Document, Node};

use crate::maze::feature::Algorithm;
use crate::maze::paint::DrawingInstructions;
use crate::maze::paint::WebColour;

#[allow(non_upper_case_globals)]
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
    pub fn new(ring_sizes: &[usize], default: T) -> Self {
        let extents = ring_sizes
            .iter()
            .scan(0, |state, &x| {
                *state += x;
                Some(*state)
            })
            .collect::<Vec<_>>();
        Self {
            ring_sizes: ring_sizes.to_vec(),
            cells: vec![default; ring_sizes.iter().sum::<usize>()],
            extents,
        }
    }

    pub fn from_iter(ring_sizes: &[usize], iter: impl Iterator<Item = T>) -> Self {
        let extents = ring_sizes
            .iter()
            .scan(0, |state, &x| {
                *state += x;
                Some(*state)
            })
            .collect::<Vec<_>>();
        Self {
            ring_sizes: ring_sizes.to_vec(),
            cells: iter.collect::<Vec<_>>(),
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
    type Idx: Eq + PartialEq + Copy + Clone;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx);

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_random_node(&self) -> Self::Idx;

    fn open(&mut self, node: Self::Idx);

    fn dijkstra(&self, origin: Self::Idx) -> Cells<usize>;

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)>;

    fn get_all_nodes(&self) -> Vec<Self::Idx>;

    fn get_indexed<T: Copy>(&self, default: T) -> Cells<T>;

    fn get_index(&self, node: Self::Idx) -> usize;
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

    fn get_all_edges(&self) -> Vec<(RingNode, RingNode)> {
        let mut frontier = vec![&self.cells[0]];
        let mut edges: Vec<(RingNode, RingNode)> = vec![];
        while !frontier.is_empty() {
            let node = frontier.pop().unwrap();
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

    fn get_indexed<T: Copy>(&self, default: T) -> Cells<T> {
        Cells::new(&self.ring_sizes, default)
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        node.column + self.ring_sizes[0..node.row].iter().sum::<usize>()
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

    /// No bounds checking on `ring`. Panics if `ring` ≥ `ring_sizes.len()` of this maze
    fn max_column(&self, ring: usize) -> usize {
        self.ring_sizes[ring]
    }
}

fn jarník<M: JarníkMaze>(mut maze: M) -> M {
    let start = maze.get_random_node();
    let mut vertices: Vec<M::Idx> = vec![start];
    let mut visited = vec![false; maze.get_all_nodes().len()];
    visited[maze.get_index(start)] = true;

    while !vertices.is_empty() {
        let i = vertices.len() - 1;
        let e = vertices[i];
        let possible_targets = maze
            .get_walls(e)
            .iter()
            .cloned()
            .filter(|n| !visited[maze.get_index(*n)])
            .collect::<Vec<_>>();
        if !possible_targets.is_empty() {
            let target = possible_targets[fastrand::usize(0..possible_targets.len())];
            maze.carve(e, target);
            visited[maze.get_index(target)] = true;
            vertices.push(target);
        } else {
            vertices.swap_remove(i);
        }
    }

    maze
}

struct Kruskal<'a, M>
where
    M: JarníkMaze,
{
    maze: &'a mut M,
    // we need a bijection between each node's class and all the nodes in a given class
    // a class is just an integer that happens to be the same one as the node's original
    // index from get_all_nodes()
    // each node has exactly one class
    // each class can have multiple nodes, but starts out with exactly one node
    class_members: Vec<Vec<M::Idx>>,
    classes2: Vec<usize>,
}

impl<'a, M> Kruskal<'a, M>
where
    M: JarníkMaze,
{
    fn new(maze: &'a mut M) -> Self {
        let all_nodes = maze.get_all_nodes().iter().cloned().collect::<Vec<_>>();
        Self {
            maze,
            classes2: all_nodes.iter().enumerate().map(|(i, _)| i).collect(),
            class_members: all_nodes
                .iter()
                .cloned()
                .map(|n| vec![n])
                .collect::<Vec<_>>(),
        }
    }

    fn link(&mut self, a: M::Idx, b: M::Idx) {
        self.maze.carve(a, b);
        let class_of_a = self.classes2[self.maze.get_index(a)];
        let class_of_b = self.classes2[self.maze.get_index(b)];
        let members_of_b = self.class_members[class_of_b].drain(..).collect::<Vec<_>>();
        for member_of_b in members_of_b {
            self.classes2[self.maze.get_index(member_of_b)] = class_of_a;
            self.class_members[class_of_a].push(member_of_b);
        }
    }

    fn classes_are_distinct(&self, a: M::Idx, b: M::Idx) -> bool {
        self.classes2[self.maze.get_index(a)] != self.classes2[self.maze.get_index(b)]
    }
}

fn kruskal<M: JarníkMaze>(mut maze: M) -> M {
    let mut edges = maze.get_all_edges();
    let mut state = Kruskal::<M>::new(&mut maze);
    fastrand::shuffle(&mut edges);

    for (a, b) in edges {
        if state.classes_are_distinct(a, b) {
            state.link(a, b);
        }
    }

    maze
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
        self.row == other.row
            && (self.column + 1 == other.column
                || (other.column == 0 && self.column == extents[self.row] - 1))
    }

    pub fn is_west_of(&self, other: RingNode, extents: &[usize]) -> bool {
        self.row == other.row
            && (self.column == other.column + 1
                || (self.column == 0 && other.column == extents[self.row] - 1))
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
}

#[derive(Debug, Clone, Copy)]
struct PolarPoint {
    θ: f64,
    r: f64,
}

impl PolarPoint {
    fn to_cartesian(self, centre: CartesianPoint) -> CartesianPoint {
        CartesianPoint {
            x: centre.x + (self.r * self.θ.cos()),
            y: centre.y + (self.r * self.θ.sin()),
        }
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

    fn compute_cartesian_coordinates(
        &self,
        inner: f64,
        outer: f64,
        east: f64,
        west: f64,
    ) -> CellCoordinates {
        CellCoordinates {
            ax: self.centre.x + (inner * west.cos()),
            ay: self.centre.y + (inner * west.sin()),
            bx: self.centre.x + (outer * west.cos()),
            by: self.centre.y + (outer * west.sin()),
            cx: self.centre.x + (inner * east.cos()),
            cy: self.centre.y + (inner * east.sin()),
            dx: self.centre.x + (outer * east.cos()),
            dy: self.centre.y + (outer * east.sin()),
        }
    }

    fn compute_cell(&self, node: RingNode) -> CellCoordinates {
        let inner = self.inner_radius(node.row);
        let outer = self.outer_radius(node.row);
        let east = self.θ_east(node);
        let west = self.θ_west(node);

        self.compute_cartesian_coordinates(inner, outer, east, west)
    }

    // make the stain cells ever so slightly bigger to avoid gaps between cells
    fn compute_cell_with_fudge(&self, node: RingNode) -> CellCoordinates {
        let inner = self.inner_radius(node.row) - 1.5;
        let outer = self.outer_radius(node.row) + 1.5;
        let fudge = self.θ(node.row) / 25.0;
        let east = self.θ_east(node) - fudge;
        let west = self.θ_west(node) + fudge;

        self.compute_cartesian_coordinates(inner, outer, east, west)
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

fn make_maze(
    rings: usize,
    column_factor: usize,
    seed: u64,
    algorithm: &Algorithm,
) -> (RingMaze, Vec<RingNode>, Cells<usize>) {
    fastrand::seed(seed);
    let template = RingMaze::new(rings, column_factor);
    let mut maze = match algorithm {
        Algorithm::GrowingTree => jarník(template),
        Algorithm::Kruskal => kruskal(template),
    };
    let start = get_random_cell_on_the_outside(&maze);
    let exit = get_node_furthest_away_from(&maze, start);
    let entrance = get_node_furthest_away_from(&maze, exit);
    let (path_to_solution, distances) = find_shortest_path(&maze, entrance, exit);
    maze.open(entrance);
    maze.open(exit);

    (maze, path_to_solution, distances)
}

/*
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
*/

fn find_shortest_path(
    maze: &RingMaze,
    start: RingNode,
    end: RingNode,
) -> (Vec<RingNode>, Cells<usize>) {
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

    (path, distances)
}

fn get_random_cell_on_the_outside(maze: &RingMaze) -> RingNode {
    let ring = maze.ring_sizes.len() - 1;
    let column = fastrand::usize(0..maze.ring_sizes[ring]);
    RingNode { row: ring, column }
}

pub trait MazeGen {
    fn create_maze(
        &self,
        seed: u64,
        features: Vec<DrawingInstructions>,
        algorithm: &Algorithm,
    ) -> String;
}

pub struct RingMazeSvg {
    pub stroke_width: f64,
    pub cell_size: f64,
    pub colour: String,
    pub size: usize,
}

impl From<CartesianPoint> for Parameters {
    fn from(val: CartesianPoint) -> Parameters {
        (val.x, val.y).into()
    }
}

impl RingMazeSvg {
    fn polar(grid: &PolarGrid, node: &RingNode) -> PolarPoint {
        if node.row == 0 && node.column == 0 {
            return PolarPoint { r: 0.0, θ: 0.0 };
        }
        PolarPoint {
            r: grid.inner_radius(node.row) + grid.ring_height / 2.0,
            θ: grid.θ_east(*node) + grid.θ(node.row) / 2.0,
        }
    }

    fn draw_path(&self, grid: &PolarGrid, path: &Vec<RingNode>, colour: WebColour) -> Path {
        let nodes = path
            .iter()
            .enumerate()
            .flat_map(|(i, node)| {
                if path[i.saturating_sub(1)].row != node.row
                    && path[min(path.len() - 1, i + 1)].row != node.row
                {
                    vec![node, node]
                } else {
                    vec![node]
                }
            })
            .collect::<Vec<_>>();

        let split_path = nodes
            .iter()
            .map(|node| Self::polar(grid, node))
            .collect::<Vec<_>>();

        let points = split_path
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let prev = split_path[i.saturating_sub(1)];
                let next = split_path[min(split_path.len() - 1, i + 1)];
                if node.r < prev.r {
                    PolarPoint {
                        θ: prev.θ,
                        r: node.r,
                    }
                } else if node.r < next.r {
                    PolarPoint {
                        θ: next.θ,
                        r: node.r,
                    }
                } else {
                    *node
                }
            })
            .map(|p| p.to_cartesian(grid.centre))
            .collect::<Vec<_>>();

        let r_out = grid.outer_radius(path[0].row) + self.stroke_width / 2.0;
        let mut data = Data::new().move_to(
            (PolarPoint {
                r: r_out,
                θ: split_path[0].θ,
            })
            .to_cartesian(grid.centre),
        );
        points.iter().enumerate().for_each(|(i, point)| {
            if split_path[i.saturating_sub(1)].r == split_path[i].r {
                let sweep =
                    if nodes[i.saturating_sub(1)].is_west_of(*nodes[i], &grid.maze.ring_sizes) {
                        0
                    } else {
                        1
                    };
                data.append(Command::EllipticalArc(
                    Absolute,
                    (
                        split_path[i].r,
                        split_path[i].r,
                        0,
                        0,
                        sweep,
                        point.x,
                        point.y,
                    )
                        .into(),
                ));
            } else {
                data.append(Command::Line(Absolute, (point.x, point.y).into()));
            }
        });
        {
            let exit = (PolarPoint {
                r: r_out,
                θ: split_path.last().unwrap().θ,
            })
            .to_cartesian(grid.centre);
            data.append(Command::Line(Absolute, (exit.x, exit.y).into()));
        };

        Path::new()
            .set("stroke", colour.to_web_string())
            .set("fill", "none")
            .set("stroke-linejoin", "round")
            .set("d", data)
            .set("stroke-width", 1.5 * self.stroke_width)
    }

    fn stain(
        &self,
        grid: &PolarGrid,
        distances: &Cells<usize>,
        (a, b): (WebColour, WebColour),
        document: &mut Document,
    ) {
        let max_distance = *distances.cells.iter().max().unwrap() as f64;
        let get_fill = |node: RingNode| {
            let intensity = (max_distance - distances[node] as f64) / max_distance;
            let inverse = 1.0 - intensity;
            a.blend(intensity).add(&b.blend(inverse)).to_web_string()
        };
        {
            document.append(
                Circle::new()
                    .set("cx", grid.centre.x)
                    .set("cy", grid.centre.y)
                    .set("r", grid.ring_height + 1.0)
                    .set("stroke", "none")
                    .set("fill", get_fill(RingNode { column: 0, row: 0 })),
            );
        };

        for node in grid.maze.cells.iter().skip(1) {
            let outer = grid.outer_radius(node.coordinates.row);
            let inner = grid.inner_radius(node.coordinates.row);
            let c = grid.compute_cell_with_fudge(node.coordinates);
            let data = Data::new()
                .move_to((c.ax, c.ay))
                .line_to((c.bx, c.by))
                .elliptical_arc_to((outer, outer, 0, 0, 0, c.dx, c.dy))
                .line_to((c.cx, c.cy))
                .elliptical_arc_to((inner, inner, 0, 0, 1, c.ax, c.ay));
            let path = Path::new()
                .set("stroke", "none")
                .set("fill", get_fill(node.coordinates))
                .set("d", data);
            document.append(path);
        }
    }

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
}

impl MazeGen for RingMazeSvg {
    fn create_maze(
        &self,
        seed: u64,
        features: Vec<DrawingInstructions>,
        algorithm: &Algorithm,
    ) -> String {
        let (maze, path_to_solution, distances) = make_maze(self.size, 8, seed, algorithm);
        let grid = PolarGrid::new(&maze, self.cell_size, self.stroke_width);
        let pixels = (grid.centre.x + self.stroke_width) * 2.0;
        let mut document = Document::new().set("viewBox", (0, 0, pixels, pixels));

        for feature in features {
            match feature {
                DrawingInstructions::ShowSolution(colour) => {
                    document.append(self.draw_path(&grid, &path_to_solution, colour));
                }
                DrawingInstructions::StainMaze(colours) => {
                    self.stain(&grid, &distances, colours, &mut document);
                }
                DrawingInstructions::DrawMaze(_) => {}
            }
        }

        let mut data = Data::new();
        for node in maze.cells.iter() {
            Self::render_cell(&mut data, &grid, node);
        }

        let path = Path::new()
            .set("stroke", self.colour.as_str())
            .set("fill", "none")
            .set("stroke-linecap", "round")
            .set("d", data)
            .set("stroke-width", self.stroke_width);
        document.append(path);

        let mut strbuf: Vec<u8> = Vec::new();
        svg::write(&mut strbuf, &document).unwrap();
        String::from_utf8(strbuf).unwrap()
    }
}

pub fn test_maze() -> Result<(), ()> {
    let mazegen = RingMazeSvg {
        cell_size: 40.0,
        size: 100,
        colour: "black".into(),
        stroke_width: 4.0,
    };
    let str = mazegen.create_maze(
        fastrand::get_seed(),
        vec![
            /*
            DrawingInstructions::StainMaze((
                WebColour {
                    r: 255,
                    g: 50,
                    b: 255,
                    a: 255,
                },
                WebColour {
                    r: 50,
                    g: 120,
                    b: 255,
                    a: 255,
                },
            )),
            DrawingInstructions::ShowSolution(WebColour {
                r: 255,
                g: 128,
                b: 255,
                a: 255,
        }),
            */
        ],
        &Algorithm::Kruskal,
    );
    // println!("{}", str);
    Ok(())
}
