use svg::node::element::path::Command::EllipticalArc;
use svg::node::element::path::Data;
use svg::node::element::path::Position::Absolute;
use svg::node::element::Path;
use svg::{Document, Node};

const π: f64 = std::f64::consts::PI;

#[derive(Copy, Clone, Debug)]
pub struct PolarNode {
    pub row: usize,
    pub column: usize,
}

const RING_HEIGHT: f64 = 40.0;

fn max_column(row: usize) -> usize {
    ((row as f64 / 2.0).ceil() as usize) * 10
}

fn θ(row: usize) -> f64 {
    // TODO: cache theta values?
    2.0 * π / max_column(row) as f64
}

fn inner_radius(row: usize) -> f64 {
    RING_HEIGHT * row as f64
}

fn outer_radius(row: usize) -> f64 {
    RING_HEIGHT * (row + 1) as f64
}

fn θ_west(node: PolarNode) -> f64 {
    θ(node.row) * (1.0 + node.column as f64)
}

fn θ_east(node: PolarNode) -> f64 {
    θ(node.row) * (node.column as f64)
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

    fn compute_cell(node: PolarNode) -> CellCoordinates {
        let inner = inner_radius(node.row);
        let outer = outer_radius(node.row);
        let east = θ_east(node);
        let west = θ_west(node);
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

    fn spoke(column: usize) -> Path {
        let node = PolarNode { column, row: 10 };
        let centre = CartesianPoint { x: 500, y: 500 };
        let cell = compute_cell(node);
        let data = Data::new()
            .move_to((centre.x, centre.y))
            .line_to((cell.bx, cell.by));
        Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("d", data)
            .set("stroke-width", "3")
    }

    let mut document = Document::new().set("viewBox", (0, 0, 1000, 1000));

    fn arc(column: usize, row: usize) -> Path {
        let node = PolarNode { column, row };
        let cell = compute_cell(node);
        let outer = outer_radius(node.row);
        let inner = inner_radius(node.row);
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

    for row in 1..10 {
        for column in 0..max_column(row) {
            document.append(arc(column, row));
        }
    }

    svg::save("test-output.svg", &document).unwrap();

    Ok(())
}
