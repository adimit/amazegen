use svg::{
    node::element::{
        path::{Command, Data, Position},
        Path,
    },
    Document, Node,
};

use crate::maze::{
    interface::{Maze, MazeRenderer, Solution},
    shape::{
        coordinates::Cartesian,
        delta::{is_top, DeltaMaze, Direction},
    },
};

use super::RenderedMaze;

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
        todo!()
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
        self.solution
            .path
            .iter()
            .map(|cell| self.compute_centre(cell))
            .for_each(|coords| data.append(Command::Line(Position::Absolute, coords.into())));
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
        let size = self.maze.size();
        let x =
            (size as f64 / 2.0) * (self.edge_length) + self.stroke_width + (self.edge_length / 2.0);
        let y = size as f64 * self.cell_height + self.stroke_width * 1.18; // ?? Why 1.18?
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
        if is_top(cell) {
            self.render_top_cell(data, cell);
        } else {
            self.render_bottom_cell(data, cell);
        }
    }

    fn render_top_cell(&self, data: &mut Data, cell: Cartesian<u32>) {
        let (x, y) = cell.get();
        let xpos = ((x as f64 / 2.0) * self.edge_length) + (self.stroke_width / 2.0);
        let ypos = y as f64 * self.cell_height + (self.stroke_width / 2.0);

        let c = |d: Direction| {
            if self.maze.has_path(&cell, d) {
                Command::Move
            } else {
                Command::Line
            }
        };

        data.append(Command::Move(Position::Absolute, (xpos, ypos).into()));
        data.append(c(Direction::ALPHA)(
            Position::Relative,
            (self.edge_length, 0).into(),
        ));
        data.append(c(Direction::EAST)(
            Position::Relative,
            (-(self.edge_length / 2.0), self.cell_height).into(),
        ));
        data.append(c(Direction::WEST)(Position::Absolute, (xpos, ypos).into()));
    }

    fn render_bottom_cell(&self, data: &mut Data, cell: Cartesian<u32>) {
        let (x, y) = cell.get();
        let xpos = ((x as f64 / 2.0) * self.edge_length) + (self.stroke_width / 2.0);
        let ypos = (1 + y) as f64 * self.cell_height + (self.stroke_width / 2.0);

        let c = |d: Direction| {
            if self.maze.has_path(&cell, d) {
                Command::Move
            } else {
                Command::Line
            }
        };

        data.append(Command::Move(Position::Absolute, (xpos, ypos).into()));
        data.append(c(Direction::ALPHA)(
            Position::Relative,
            (self.edge_length, 0).into(),
        ));
        data.append(c(Direction::EAST)(
            Position::Relative,
            (-(self.edge_length / 2.0), -self.cell_height).into(),
        ));
        data.append(c(Direction::WEST)(Position::Absolute, (xpos, ypos).into()));
    }

    fn compute_centre(&self, cell: &Cartesian<u32>) -> (f64, f64) {
        let (x, y) = cell.get();
        let xpos = (((x as f64 / 2.0) + 0.5) * self.edge_length) + (self.stroke_width / 2.0);
        let ypos = (y as f64 + if is_top(*cell) { 1.0 - q } else { q }) * self.cell_height
            + (self.stroke_width / 2.0);
        (xpos, ypos)
    }
}
const q: f64 = 1.61803398875 - 1.0;
