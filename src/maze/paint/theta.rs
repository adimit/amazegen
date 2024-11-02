use std::cmp::min;
use svg::node::element::path::Command::{self, EllipticalArc};
use svg::node::element::path::Position::Absolute;
use svg::node::element::Circle;
use svg::node::element::{
    path::{Data, Parameters},
    Path,
};
use svg::{Document, Node};

use crate::maze::feature::Svg;
use crate::maze::interface::{MazeRenderer, Solution};
use crate::maze::shape::theta::{RingCell, RingMaze, RingNode};

use super::svg::write_document;
use super::{Gradient, WebColour};

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

pub struct RingMazeRenderer<'a> {
    solution: &'a Solution<RingNode>,
    stroke_width: f64,
    document: Document,
    grid: PolarGrid<'a>,
}

impl<'a> RingMazeRenderer<'a> {
    pub fn new(
        maze: &'a RingMaze,
        path: &'a Solution<RingNode>,
        stroke_width: f64,
        cell_size: f64,
    ) -> Self {
        let grid = PolarGrid::new(maze, cell_size, stroke_width);
        let pixels = (grid.centre.x + stroke_width) * 2.0;
        let document = Document::new().set("viewBox", (0, 0, pixels, pixels));

        RingMazeRenderer {
            solution: path,
            stroke_width,
            document,
            grid,
        }
    }

    fn polar(&self, node: &RingNode) -> PolarPoint {
        if node.row == 0 && node.column == 0 {
            return PolarPoint { r: 0.0, θ: 0.0 };
        }
        PolarPoint {
            r: self.grid.inner_radius(node.row) + self.grid.ring_height / 2.0,
            θ: self.grid.θ_east(*node) + self.grid.θ(node.row) / 2.0,
        }
    }

    fn render_cell(data: &mut Data, grid: &PolarGrid, cell: &RingCell) {
        let node = cell.coordinates;
        let c = grid.compute_cell(node);
        let outer = grid.outer_radius(node.row);
        let inner = grid.inner_radius(node.row);

        // western wall
        if cell
            .inaccessible_neighbours
            .iter()
            .any(|it| it.is_west_of(node, &grid.maze.ring_sizes))
        {
            data.append(Command::Move(Absolute, (c.cx, c.cy).into()));
            data.append(Command::Line(Absolute, (c.dx, c.dy).into()));
        }

        // northern wall (only if we're on the outer ring)
        if cell.coordinates.row == grid.maze.ring_sizes.len() - 1
            && !cell
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

        // southern wall
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

    fn split_nodes_traversing_north(&self) -> Vec<&RingNode> {
        self.solution
            .path
            .iter()
            .enumerate()
            .flat_map(|(i, node)| {
                if self.solution.path[i.saturating_sub(1)].row != node.row
                    && self.solution.path[min(self.solution.path.len() - 1, i + 1)].row != node.row
                {
                    vec![node, node]
                } else {
                    vec![node]
                }
            })
            .collect::<Vec<_>>()
    }
}

impl MazeRenderer<RingMaze> for RingMazeRenderer<'_> {
    fn stain(&mut self, gradient: (WebColour, WebColour)) {
        let gradient = Gradient::new(gradient, self.grid.maze, self.solution);
        {
            self.document.append(
                Circle::new()
                    .set("cx", self.grid.centre.x)
                    .set("cy", self.grid.centre.y)
                    .set("r", self.grid.ring_height + 1.0)
                    .set("stroke", "none")
                    .set(
                        "fill",
                        gradient
                            .compute(&RingNode { column: 0, row: 0 })
                            .to_web_string(),
                    ),
            );
        };

        for node in self.grid.maze.cells.iter().skip(1) {
            let outer = self.grid.outer_radius(node.coordinates.row);
            let inner = self.grid.inner_radius(node.coordinates.row);
            let c = self.grid.compute_cell_with_fudge(node.coordinates);
            let data = Data::new()
                .move_to((c.ax, c.ay))
                .line_to((c.bx, c.by))
                .elliptical_arc_to((outer, outer, 0, 0, 0, c.dx, c.dy))
                .line_to((c.cx, c.cy))
                .elliptical_arc_to((inner, inner, 0, 0, 1, c.ax, c.ay));
            let path = Path::new()
                .set("stroke", "none")
                .set("fill", gradient.compute(&node.coordinates).to_web_string())
                .set("d", data);
            self.document.append(path);
        }
    }

    fn solve(&mut self, stroke_colour: WebColour) {
        let nodes = self.split_nodes_traversing_north();

        let polar_points = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let path_prev = nodes[i.saturating_sub(1)];
                let path_next = nodes[min(nodes.len() - 1, i + 1)];
                if node.row < path_prev.row {
                    PolarPoint {
                        r: self.grid.inner_radius(node.row) + self.grid.ring_height / 2.0,
                        θ: self.grid.θ_east(*path_prev) + self.grid.θ(path_prev.row) / 2.0,
                    }
                } else if node.row < path_next.row {
                    PolarPoint {
                        r: self.grid.inner_radius(node.row) + self.grid.ring_height / 2.0,
                        θ: self.grid.θ_east(*path_next) + self.grid.θ(path_next.row) / 2.0,
                    }
                } else {
                    self.polar(node)
                }
            })
            .collect::<Vec<_>>();

        let cartesian_points = polar_points
            .iter()
            .map(|p| p.to_cartesian(self.grid.centre))
            .collect::<Vec<_>>();

        let r_out = self.grid.outer_radius(self.solution.path[0].row) + self.stroke_width / 2.0;
        let mut data = Data::new()
            .move_to(
                (PolarPoint {
                    r: r_out - self.stroke_width / 3.0,
                    θ: polar_points[0].θ,
                })
                .to_cartesian(self.grid.centre),
            )
            .line_to::<CartesianPoint>(cartesian_points[0]);

        cartesian_points
            .into_iter()
            .enumerate()
            // we already drew the line from the outside to the first node when creating data
            .skip(1)
            .for_each(|(i, point)| {
                let prev_i = i.saturating_sub(1);
                if nodes[prev_i].row == nodes[i].row {
                    let sweep = if (polar_points[prev_i].θ > polar_points[i].θ
                        && !(nodes[prev_i].column > 2 && nodes[i].column == 0))
                        || (nodes[prev_i].column == 0 && nodes[i].column > 2)
                    {
                        0
                    } else {
                        1
                    };
                    data.append(Command::EllipticalArc(
                        Absolute,
                        (
                            polar_points[i].r,
                            polar_points[i].r,
                            0,
                            0,
                            sweep,
                            point.x,
                            point.y,
                        )
                            .into(),
                    ));
                } else {
                    data.append(Command::Line(Absolute, point.into()));
                }
            });
        {
            let exit = (PolarPoint {
                r: r_out - self.stroke_width / 3.0,
                θ: polar_points.last().unwrap().θ,
            })
            .to_cartesian(self.grid.centre);
            data.append(Command::Line(Absolute, (exit.x, exit.y).into()));
        };

        let p = Path::new()
            .set("stroke", stroke_colour.to_web_string())
            .set("fill", "none")
            .set("stroke-linejoin", "round")
            .set("d", data)
            .set("stroke-width", 1.5 * self.stroke_width);

        self.document.append(p);
    }

    fn paint(&mut self, border: WebColour) {
        let mut data = Data::new();
        for node in self.grid.maze.cells.iter() {
            Self::render_cell(&mut data, &self.grid, node);
        }

        let path = Path::new()
            .set("stroke", border.to_web_string())
            .set("fill", "none")
            .set("stroke-linecap", "round")
            .set("d", data)
            .set("stroke-width", self.stroke_width);
        self.document.append(path);
    }

    fn render(&self) -> Svg {
        write_document(&self.document)
    }
}
