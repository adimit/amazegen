use svg::{
    node::element::path::{Command, Data, Position::Absolute, Position::Relative},
    Node,
};

use crate::maze::{
    interface::{Maze, MazeRenderer, Solution},
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
    fn stain(&mut self, gradient: (WebColour, WebColour)) {
        todo!()
    }

    fn solve(&mut self, stroke_colour: WebColour) {
        todo!()
    }

    fn paint(&mut self, border: WebColour) {
        let mut data = Data::new();
        self.maze
            .get_all_nodes()
            .iter()
            .for_each(|cell| self.render_cell(&mut data, *cell));
        let path = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", border.to_web_string())
            .set("stroke-width", self.stroke_width)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("d", data);
        self.document.append(path);
    }

    fn render(&self) -> crate::maze::feature::Svg {
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
    s: f64,
    a: f64,
    b: f64,
    cell_height: f64,
    cell_width: f64,
}

impl Dimensions {
    fn new(cell_width: f64) -> Self {
        let s = cell_width / 2.0;
        let a = s / 2.0;
        let b = s * 3.0_f64.sqrt() / 2.0;
        let cell_height = b * 2.0;

        Self {
            s,
            a,
            b,
            cell_height,
            cell_width,
        }
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
            maze_size * cell_width + stroke_width,
            maze_size * dim.cell_height + dim.cell_height / 2.0 + stroke_width * 2.0,
        );

        Self {
            maze,
            solution,
            stroke_width,
            dimensions: dim,
            document: svg::Document::new().set("viewBox", (0, 0, x, y)),
        }
    }

    fn render_cell(&self, data: &mut Data, cell: Cartesian) {
        let Dimensions {
            a,
            b,
            cell_height,
            cell_width,
            ..
        } = self.dimensions;
        let x = cell_width * cell.x() as f64 - cell.x() as f64 * a + self.stroke_width;
        let y = cell_height * cell.y() as f64
            + if cell.x() % 2 == 0 { b } else { 2.0 * b }
            + self.stroke_width;

        data.append(Command::Move(Absolute, (x, y).into()));

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
        data.append(c(Direction::SouthEast)(Relative, (-a, b).into()));
        data.append(c(Direction::South)(Relative, (2.0 * -a, 0).into()));
        data.append(c(Direction::SouthWest)(Relative, (-a, -b).into()));
    }
}
