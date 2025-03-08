use svg::{
    node::element::{
        path::{Command, Data, Position::Absolute, Position::Relative},
        Path,
    },
    Node,
};

use crate::maze::{
    interface::{Maze, MazeRenderer, Metadata, Solution},
    paint::Gradient,
    shape::sigma::{Cartesian, Direction, SigmaMaze},
};

use super::{svg::write_document, WebColour};

pub struct SigmaMazeRenderer<'a> {
    maze: &'a SigmaMaze,
    solution: &'a Solution<Cartesian>,
    stroke_width: f64,
    dimensions: Dimensions,
    document: svg::Document,
}

impl MazeRenderer<SigmaMaze> for SigmaMazeRenderer<'_> {
    fn stain(&mut self, colours: (WebColour, WebColour)) {
        let gradient = Gradient::new(colours, self.maze, self.solution);
        self.maze.get_all_nodes().iter().for_each(|cell| {
            let (x, y) = self.compute_centre(cell);
            let af = self.dimensions.a + 1.0;
            let bf = self.dimensions.b + 1.0;
            let data = Data::new()
                .move_to((x - 2.0 * af, y))
                .line_by((af, -bf))
                .line_by((2.0 * af, 0))
                .line_by((af, bf))
                .line_by((-af, bf))
                .line_by((-2.0 * af, 0))
                .line_by((-af, -bf));
            let hex = Path::new()
                .set("fill", gradient.compute(cell).to_web_string())
                .set("stroke", "none")
                .set("d", data);
            self.document.append(hex);
        });
    }

    fn solve(&mut self, stroke_colour: WebColour) {
        let mut data = Data::new();
        let entrance = {
            let (x, y) = self.compute_centre(&self.solution.path[0]);
            let neighbours = self.maze.cells[self.maze.get_index(self.solution.path[0])]
                .accessible
                .clone();
            let Dimensions { a, b, .. } = self.dimensions;
            if neighbours[Direction::NorthWest].is_some() && self.solution.path[0].x() % 2 == 0 {
                (x - (a * 1.5), y - (b / 2.0))
            } else if neighbours[Direction::NorthEast].is_some()
                && self.solution.path[0].x() % 2 == 0
            {
                (x + (a * 1.5), y - (b / 2.0))
            } else {
                (x, y - b + self.stroke_width / 2.0)
            }
        };

        let exit = {
            let last = self.solution.path.last().unwrap();
            let (x, y) = self.compute_centre(last);
            let neighbours = self.maze.cells[self.maze.get_index(*last)]
                .accessible
                .clone();
            let Dimensions { a, b, .. } = self.dimensions;
            if neighbours[Direction::SouthWest].is_some() && last.x() % 2 == 1 {
                (x - (a * 1.5), y + (b / 2.0))
            } else if neighbours[Direction::SouthEast].is_some() && last.x() % 2 == 1 {
                (x + (a * 1.5), y + (b / 2.0))
            } else {
                (x, y + b - self.stroke_width / 2.0)
            }
        };

        data.append(Command::Move(Absolute, entrance.into()));
        self.solution
            .path
            .iter()
            .map(|node| self.compute_centre(node))
            .for_each(|coords| data.append(Command::Line(Absolute, coords.into())));
        data.append(Command::Line(Absolute, exit.into()));

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", stroke_colour.to_web_string())
            .set("stroke-width", self.stroke_width * 2.0)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("d", data);
        self.document.append(path);
    }

    fn paint(&mut self, border: WebColour) {
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

    fn render(&mut self, metadata: &Metadata) -> crate::maze::feature::Svg {
        metadata.append_to_svg_document(&mut self.document);
        write_document(&self.document)
    }
}

/// `a` is the length of a side of the hexagon
/// in the upper left corner of a rectangle containing the hexagon, `s` is
/// the hypothenuse of a triangle of which `a` and `b` are the legs in the
/// horizontal and vertical direction, respectively.
/// The centre of the hexagon sits at (2a, b), the dimenions of the containing
/// rectangle are (2s, 2b). Note that 2a = s.
#[derive(Debug)]
struct Dimensions {
    a: f64,
    b: f64,
    cell_height: f64,
}

impl Dimensions {
    fn new(cell_width: f64) -> Self {
        let s = cell_width / 2.0;
        let a = s / 2.0;
        let b = s * 3.0_f64.sqrt() / 2.0;
        let cell_height = b * 2.0;

        Self { a, b, cell_height }
    }
}

impl<'a> SigmaMazeRenderer<'a> {
    pub fn new(
        maze: &'a SigmaMaze,
        solution: &'a Solution<Cartesian>,
        stroke_width: f64,
        cell_width: f64,
    ) -> Self {
        let dim = Dimensions::new(cell_width);
        let maze_size = maze.size() as f64;
        let (x, y) = (
            maze_size * (3.0 * dim.a) + dim.a + stroke_width,
            maze_size * dim.cell_height + dim.b + stroke_width,
        );

        Self {
            maze,
            solution,
            stroke_width,
            dimensions: dim,
            document: svg::Document::new().set("viewBox", (0, 0, x, y)),
        }
    }

    fn compute_centre(&self, cell: &Cartesian) -> (f64, f64) {
        let Dimensions {
            a, b, cell_height, ..
        } = self.dimensions;

        let x = 3.0 * a * cell.x() as f64 + 2.0 * a + (self.stroke_width / 2.0);
        let y = cell_height * cell.y() as f64
            + if cell.x() % 2 == 0 { b } else { 2.0 * b }
            + (self.stroke_width / 2.0);

        (x, y)
    }

    fn render_cell(&self, data: &mut Data, cell: Cartesian) {
        let Dimensions { a, b, .. } = self.dimensions;
        let (x, y) = self.compute_centre(&cell);

        data.append(Command::Move(Absolute, (x - 2.0 * a, y).into()));

        let c = |d: Direction| {
            if self.maze.has_path(&cell, d) {
                Command::Move
            } else {
                Command::Line
            }
        };

        data.append(c(Direction::NorthWest)(Relative, (a, -b).into()));
        data.append(c(Direction::North)(Relative, (2.0 * a, 0).into()));
        data.append(c(Direction::NorthEast)(Relative, (a, b).into()));
        if cell.x() == 0 || cell.y() == self.maze.size() - 1 || cell.x() == self.maze.size() - 1 {
            data.append(c(Direction::SouthEast)(Relative, (-a, b).into()));
            data.append(c(Direction::South)(Relative, (-2.0 * a, 0).into()));
            data.append(c(Direction::SouthWest)(Relative, (-a, -b).into()));
        }
    }
}
