use std::cmp::min;
use svg::node::element::path::Command::{self, EllipticalArc};
use svg::node::element::path::Position::Absolute;
use svg::node::element::Circle;
use svg::{Document, Node};

use svg::node::element::{
    path::{Data, Parameters},
    Path,
};

use crate::maze::interface::{Maze, MazePath, MazeToSvg};
use crate::maze::theta::{RingCell, RingMaze, RingNode};

use super::{DrawingInstructions, WebColour};

#[allow(non_upper_case_globals)]
const π: f64 = std::f64::consts::PI;

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

pub struct PolarGrid<'a> {
    ring_height: f64,
    maze: &'a RingMaze,
    pub centre: CartesianPoint,
}

impl PolarGrid<'_> {
    pub fn new(maze: &RingMaze, ring_height: f64, stroke_width: f64) -> PolarGrid {
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
    pub x: f64,
    pub y: f64,
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

impl From<CartesianPoint> for Parameters {
    fn from(val: CartesianPoint) -> Parameters {
        (val.x, val.y).into()
    }
}

pub struct RingMazePainter {
    pub stroke_width: f64,
    pub cell_size: f64,
    pub colour: String,
}

impl RingMazePainter {
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
        distances: &[usize],
        (a, b): (WebColour, WebColour),
        document: &mut Document,
    ) {
        let max_distance = *distances.iter().max().unwrap() as f64;
        let get_fill = |node: RingNode| {
            let intensity =
                (max_distance - distances[grid.maze.get_index(node)] as f64) / max_distance;
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

impl MazeToSvg<RingMaze> for RingMazePainter {
    fn paint_maze(
        &self,
        features: Vec<DrawingInstructions>,
        maze: &RingMaze,
        path: &MazePath<RingNode>,
    ) -> String {
        let grid = PolarGrid::new(maze, self.cell_size, self.stroke_width);
        let pixels = (grid.centre.x + self.stroke_width) * 2.0;
        let mut document = Document::new().set("viewBox", (0, 0, pixels, pixels));

        for feature in features {
            match feature {
                DrawingInstructions::ShowSolution(colour) => {
                    document.append(self.draw_path(&grid, &path.path, colour));
                }
                DrawingInstructions::StainMaze(colours) => {
                    self.stain(&grid, &path.distances, colours, &mut document);
                }
                DrawingInstructions::DrawMaze(_) => {}
            }
        }

        {
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
        }

        let mut strbuf: Vec<u8> = Vec::new();
        svg::write(&mut strbuf, &document).unwrap();
        String::from_utf8(strbuf).unwrap()
    }
}
