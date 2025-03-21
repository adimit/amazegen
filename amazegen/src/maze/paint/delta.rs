use svg::{
    node::element::{
        path::{Command, Data, Parameters, Position},
        Path,
    },
    Document, Node,
};

use crate::maze::{
    interface::{Maze, MazeRenderer, Solution},
    paint::Gradient,
    shape::{
        coordinates::Cartesian,
        delta::{is_top, DeltaMaze, Direction},
    },
};

use super::{midpoint, RenderedMaze};

pub struct DeltaMazeRenderer<'a> {
    maze: &'a DeltaMaze,
    solution: &'a Solution<Cartesian<u32>>,
    stroke_width: f64,
    edge_length: f64, // the triangles are equilateral
    cell_height: f64,
    document: Document,
}

impl MazeRenderer<DeltaMaze> for DeltaMazeRenderer<'_> {
    fn stain(&mut self, gradient: (super::WebColour, super::WebColour)) {
        let gradient = Gradient::new(gradient, self.maze, self.solution);
        self.maze.get_all_nodes().iter().for_each(|cell| {
            let Geometry { start, movements } = self.get_geometry(cell);
            let is_top_factor = if is_top(*cell) { 1.0 } else { -1.0 };
            let fudge = 2.0;
            let fudged_start = (start.0 - fudge, start.1 - (fudge * is_top_factor));
            let fudged_movements = vec![
                (
                    movements[0].1 + fudge,
                    movements[0].2 - (fudge * is_top_factor),
                ),
                (movements[1].1, movements[1].2 + (fudge * is_top_factor)),
                (
                    movements[2].1 - fudge,
                    movements[2].2 - (fudge * is_top_factor),
                ),
            ];
            let mut data = Data::new().move_to(Into::<Parameters>::into(fudged_start));
            for coords in fudged_movements.into_iter() {
                data = data.line_to(Into::<Parameters>::into(coords));
            }
            let path = Path::new()
                .set("fill", gradient.compute(cell).to_web_string())
                .set("stroke", "none")
                .set("d", data);
            self.document.append(path);
        });
    }

    fn solve(&mut self, stroke_colour: super::WebColour) {
        let mut data = Data::new();
        let entrance = {
            let (x, _) = self.compute_centre(&self.solution.path[0]);
            (x, 0)
        };
        let exit = {
            let (x, y) = self.compute_centre(self.solution.path.last().unwrap());
            (x, y + (self.cell_height / 2.0))
        };
        data.append(Command::Move(Position::Absolute, entrance.into()));
        for i in 1..self.solution.path.len() - 2 {
            let prev = self.solution.path[i - 1];
            let cur = self.solution.path[i];
            let next = self.solution.path[i + 1];

            let inbound = midpoint(self.compute_centre(&prev), self.compute_centre(&cur));
            let outbound = midpoint(self.compute_centre(&cur), self.compute_centre(&next));
            data.append(Command::Line(Position::Absolute, inbound.into()));
            data.append(Command::Line(Position::Absolute, outbound.into()));
        }
        data.append(Command::Line(Position::Absolute, exit.into()));
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", stroke_colour.to_web_string())
            .set("stroke-width", self.stroke_width)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("d", data);
        self.document.append(path);
    }

    fn paint(&mut self, border: super::WebColour) {
        let mut data = Data::new();
        self.maze
            .get_all_nodes()
            .iter()
            .for_each(|cell| self.render_cell(&mut data, *cell));
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", border.to_web_string())
            .set("stroke-width", self.stroke_width)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("d", data);
        self.document.append(path);
    }

    fn render(self) -> super::RenderedMaze {
        let (width, height) = self.maze.get_size();
        let x = (width as f64 / 2.0) * (self.edge_length)
            + self.stroke_width
            + (self.edge_length / 2.0);
        let y = height as f64 * self.cell_height + self.stroke_width;
        RenderedMaze::new(self.document, (x as u32, y.floor() as u32))
    }
}

impl<'a> DeltaMazeRenderer<'a> {
    pub fn new(
        maze: &'a DeltaMaze,
        solution: &'a Solution<Cartesian<u32>>,
        stroke_width: f64,
        edge_length: f64,
    ) -> Self {
        Self {
            maze,
            solution,
            stroke_width,
            edge_length,
            cell_height: (3.0 as f64).sqrt() / 2.0 * edge_length,
            document: Document::new(),
        }
    }

    fn render_cell(&self, data: &mut Data, cell: Cartesian<u32>) {
        let Geometry { start, movements } = self.get_geometry(&cell);
        data.append(Command::Move(Position::Absolute, start.into()));
        movements.iter().for_each(|(d, x, y)| {
            if self.maze.has_path(&cell, *d) {
                data.append(Command::Move(Position::Absolute, (*x, *y).into()));
            } else {
                data.append(Command::Line(Position::Absolute, (*x, *y).into()));
            }
        });
    }

    fn get_geometry(&self, cell: &Cartesian<u32>) -> Geometry<Direction> {
        let (x, y) = cell.get();
        let x_start = ((x as f64 / 2.0) * self.edge_length) + (self.stroke_width / 2.0);
        let y_start = if is_top(*cell) { y } else { y + 1 } as f64 * self.cell_height
            + (self.stroke_width / 2.0);
        let movements = vec![
            (Direction::ALPHA, x_start + self.edge_length, y_start),
            (
                Direction::EAST,
                x_start + (self.edge_length / 2.0),
                if is_top(*cell) {
                    y_start + self.cell_height
                } else {
                    y_start - self.cell_height
                },
            ),
            (Direction::WEST, x_start, y_start),
        ];

        Geometry {
            start: (x_start, y_start),
            movements,
        }
    }

    fn compute_centre(&self, cell: &Cartesian<u32>) -> (f64, f64) {
        let (x, y) = cell.get();
        let xpos = (((x as f64 / 2.0) + 0.5) * self.edge_length) + (self.stroke_width / 2.0);
        let ypos = (y as f64 + 0.5) * self.cell_height + (self.stroke_width / 2.0);
        (xpos, ypos)
    }
}

struct Geometry<Direction>
where
    Direction: Copy,
{
    start: (f64, f64),
    movements: Vec<(Direction, f64, f64)>,
}
