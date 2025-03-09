use svg::{
    node::element::{
        path::{
            Command, Data,
            Position::{Absolute, Relative},
        },
        Path,
    },
    Node,
};

use crate::maze::{
    interface::{Maze, MazeRenderer, Solution},
    paint::Gradient,
    shape::regular::{Direction, RectilinearMaze},
};

use super::{CellSize, RenderedMaze};

pub struct RectilinearRenderer<'a> {
    maze: &'a RectilinearMaze,
    solution: &'a Solution<(usize, usize)>,
    stroke_width: f64,
    document: svg::Document,
    cell_size: CellSize,
}

impl MazeRenderer<RectilinearMaze> for RectilinearRenderer<'_> {
    fn stain(&mut self, gradient: (super::WebColour, super::WebColour)) {
        let gradient = Gradient::new(gradient, self.maze, self.solution);
        let s = self.cell_size.0;
        let b = self.stroke_width.floor() as usize;
        for (x, y) in self.maze.get_all_nodes() {
            let rect = svg::node::element::Rectangle::new()
                .set("x", x * s + b)
                .set("y", y * s + b)
                .set("width", s)
                .set("height", s)
                .set("fill", gradient.compute(&(x, y)).to_web_string());
            self.document.append(rect);
        }
    }

    fn solve(&mut self, stroke_colour: super::WebColour) {
        let mut data = Data::new();
        let s = self.cell_size.0;
        let stroke: usize = self.stroke_width.floor() as usize;
        data.append(Command::Move(
            Absolute,
            (
                self.solution.path.first().unwrap_or(&(0, 0)).0 * s + s / 2 + stroke,
                0,
            )
                .into(),
        ));
        self.solution.path.iter().for_each(|(x, y)| {
            data.append(Command::Line(
                Absolute,
                (x * s + s / 2 + stroke, *y * s + s / 2 + stroke).into(),
            ))
        });
        data.append(Command::Line(Relative, (0, s / 2).into()));
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", stroke_colour.to_web_string())
            .set("stroke-width", self.stroke_width * 2.0)
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
            .set("stroke-width", self.stroke_width * 2.0)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("d", data);
        self.document.append(path);
    }

    fn render(&self) -> RenderedMaze {
        let (x, y) = (
            (self.maze.extents.0 * self.cell_size.0) as f64 + 2.0 * self.stroke_width,
            (self.maze.extents.1 * self.cell_size.0) as f64 + 2.0 * self.stroke_width,
        );
        RenderedMaze::new(self.document.clone(), (x.floor() as u32, y.floor() as u32))
    }
}

impl<'a> RectilinearRenderer<'a> {
    pub fn new(
        maze: &'a RectilinearMaze,
        solution: &'a Solution<(usize, usize)>,
        stroke_width: f64,
        cell_width: usize,
    ) -> Self {
        let document = svg::Document::new();

        Self {
            maze,
            solution,
            stroke_width,
            document,
            cell_size: CellSize(cell_width),
        }
    }

    fn render_cell(&self, data: &mut Data, (x, y): (usize, usize)) {
        let s = self.cell_size.0 as i32;

        let c = |d: Direction| {
            if self.maze.has_wall((x, y), d) {
                Command::Line
            } else {
                Command::Move
            }
        };

        data.append(Command::Move(
            Absolute,
            (
                x as f64 * s as f64 + self.stroke_width,
                y as f64 * s as f64 + self.stroke_width,
            )
                .into(),
        ));

        data.append(c(Direction::Up)(Relative, (s, 0).into()));
        data.append(c(Direction::Right)(Relative, (0, s).into()));
        data.append(c(Direction::Down)(Relative, (-s, 0).into()));
        data.append(c(Direction::Left)(Relative, (0, -s).into()));
    }
}
