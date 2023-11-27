use crate::maze::paint::*;
use crate::maze::regular::RectilinearMaze;
use itertools::Itertools;

use super::algorithms::{jarník, kruskal};
use super::interface::{Maze, MazeToSvg};
use super::paint::regular::RegularMazePainter;
use super::paint::theta::RingMazePainter;
use super::theta::RingMaze;

const STAIN_A: &str = "FFDC80";
const STAIN_B: &str = "B9327D";
const SOLUTION: &str = "8FE080";

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Shape {
    Rectilinear(usize, usize),
    Theta(usize),
}

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum Feature {
    Stain,
    Solve,
}

impl From<Feature> for DrawingInstructions {
    fn from(value: Feature) -> Self {
        match value {
            Feature::Stain => DrawingInstructions::StainMaze((
                WebColour::from_string(STAIN_A).unwrap(),
                WebColour::from_string(STAIN_B).unwrap(),
            )),
            Feature::Solve => {
                DrawingInstructions::ShowSolution(WebColour::from_string(SOLUTION).unwrap())
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Algorithm {
    Kruskal,
    GrowingTree,
}

impl Algorithm {
    pub fn execute<M: Maze>(&self, maze: M) -> M {
        match self {
            Algorithm::Kruskal => jarník(maze),
            Algorithm::GrowingTree => kruskal(maze),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Configuration {
    pub seed: u64,
    pub shape: Shape,
    pub colour: String,
    pub features: Vec<Feature>,
    pub algorithm: Algorithm,
    pub stroke_width: f64,
}

pub struct Svg(pub String);

impl Configuration {
    fn render_maze<M: Maze, P: MazeToSvg<M>>(&self, template: M, painter: P) -> Svg {
        let mut maze = self.algorithm.execute(template);
        let path = maze.find_path();
        Svg(painter.paint_maze(
            self.features
                .iter()
                .map(|f| Into::<DrawingInstructions>::into(*f))
                .sorted()
                .collect::<Vec<_>>(),
            &maze,
            &path,
        ))
    }

    pub fn execute(&self) -> Svg {
        fastrand::seed(self.seed);
        match self.shape {
            Shape::Rectilinear(x, y) => {
                let painter = RegularMazePainter {
                    cell_size: 40,
                    colour: self.colour.to_string(),
                    stroke_width: (self.stroke_width / 2.0).floor() as usize,
                };
                let template = RectilinearMaze::new((x, y));
                self.render_maze(template, painter)
            }
            Shape::Theta(size) => {
                let mazegen = RingMazePainter {
                    cell_size: 40.0,
                    colour: format!("#{}", self.colour.clone()),
                    stroke_width: self.stroke_width,
                };
                let template = RingMaze::new(size, 8);
                self.render_maze(template, mazegen)
            }
        }
    }
}
