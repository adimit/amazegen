use svg::node::element::path::Command::EllipticalArc;
use svg::node::element::path::Data;
use svg::node::element::path::Position::Absolute;
use svg::node::element::Path;
use svg::{Document, Node};

const π: f64 = std::f64::consts::PI;

struct RingMaze {
    rings: Vec<usize>,
}

impl RingMaze {
    fn columns(row: usize, column_factor: usize) -> usize {
        2_usize.pow(row.ilog(2)) * column_factor
    }

    pub fn new(column_factor: usize, max_rings: usize) -> RingMaze {
        let mut rings = vec![1];
        rings.extend(
            (1..max_rings)
                .into_iter()
                .map(|row| Self::columns(row, column_factor)),
        );
        RingMaze { rings }
    }

    /// No bounds checking on `ring`. Panics if `ring` > `rings` of this maze
    pub fn max_column(&self, ring: usize) -> usize {
        self.rings[ring]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PolarNode {
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

    fn θ_west(&self, node: PolarNode) -> f64 {
        self.θ(node.row) * (1.0 + node.column as f64)
    }

    fn θ_east(&self, node: PolarNode) -> f64 {
        self.θ(node.row) * (node.column as f64)
    }

    fn compute_cell(&self, node: PolarNode) -> CellCoordinates {
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

pub fn test_maze() -> Result<(), ()> {
    println!("Generating maze...");
    let maze = RingMaze::new(8, 10);
    let grid = PolarGrid::new(&maze, 40.0);

    let mut document = Document::new().set("viewBox", (0, 0, 1000, 1000));

    fn arc(grid: &PolarGrid, column: usize, row: usize) -> Path {
        let node = PolarNode { column, row };
        let cell = grid.compute_cell(node);
        let outer = grid.outer_radius(node.row);
        let inner = grid.inner_radius(node.row);
        let data = Data::new()
            .move_to((cell.ax, cell.ay))
            .line_to((cell.bx, cell.by))
            .add(EllipticalArc(
                Absolute,
                (outer, outer, 0, 0, 0, cell.dx, cell.dy).into(),
            ))
            .line_to((cell.cx, cell.cy))
            .add(EllipticalArc(
                Absolute,
                (inner, inner, 0, 1, 0, cell.ax, cell.ay).into(),
            ));

        Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("d", data)
            .set("stroke-width", "3")
    }

    for row in 1..maze.rings.len() {
        for column in 0..maze.max_column(row) {
            document.append(arc(&grid, column, row));
        }
    }

    svg::save("test-output.svg", &document).unwrap();

    Ok(())
}
