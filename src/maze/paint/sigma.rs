use crate::maze::{
    interface::{MazeRenderer, Solution},
    shape::sigma::{Cartesian, SigmaMaze},
};

pub struct SigmaMazeRenderer<'a> {
    maze: &'a SigmaMaze,
    solution: &'a Solution<Cartesian>,
    stroke_width: f64,
    cell_size: f64,
    document: svg::Document,
}

impl MazeRenderer<SigmaMaze> for SigmaMazeRenderer<'_> {
    fn stain(&mut self, gradient: (super::WebColour, super::WebColour)) {
        todo!()
    }

    fn solve(&mut self, stroke_colour: super::WebColour) {
        todo!()
    }

    fn paint(&mut self, border: super::WebColour) {
        todo!()
    }

    fn render(&self) -> crate::maze::feature::Svg {
        todo!()
    }
}

impl<'a> SigmaMazeRenderer<'a> {
    pub fn new(
        maze: &'a SigmaMaze,
        solution: &'a Solution<Cartesian>,
        stroke_width: f64,
        cell_size: f64,
    ) -> Self {
        let a = cell_size / 3.0;
        // let b =
        let pixels = maze.size() as f64 * cell_size;
        Self {
            maze,
            solution,
            stroke_width,
            cell_size,
            document: svg::Document::new().set("viewBox", (0, 0, pixels, pixels)),
        }
    }
}
