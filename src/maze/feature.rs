use itertools::Itertools;

use crate::configuration::Seed;
use crate::maze::interface::MazeRenderer;
use crate::maze::paint::theta::RingMazeRenderer;
use crate::maze::paint::*;
use crate::maze::shape::regular::RectilinearMaze;

use super::algorithms::{jarník, kruskal};
use super::interface::{Maze, Solution};
use super::paint::regular::RectilinearRenderer;
use super::paint::sigma::SigmaMazeRenderer;
use super::shape::sigma::SigmaMaze;
use super::shape::theta::RingMaze;

const STAIN_A: &str = "FFDC80";
const STAIN_B: &str = "B9327D";
const SOLUTION: &str = "8FE080";

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum Shape {
    Rectilinear(usize, usize),
    Theta(usize),
    Sigma(usize),
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Rectilinear(10, 10)
    }
}

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum Feature {
    Stain,
    Solve,
}

impl PartialOrd for Feature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Feature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        use Feature::*;
        match (self, other) {
            (Stain, Solve) => Less,
            (Solve, Stain) => Greater,
            _ => Equal,
        }
    }
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum Algorithm {
    Kruskal,
    GrowingTree,
}

impl Algorithm {
    pub fn execute<M: Maze>(&self, maze: M) -> M {
        match self {
            Algorithm::Kruskal => kruskal(maze),
            Algorithm::GrowingTree => jarník(maze),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Parameters {
    pub seed: u64,
    pub shape: Shape,
    pub colour: String,
    pub features: Vec<Feature>,
    pub algorithm: Algorithm,
    pub stroke_width: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Parameters2 {
    pub seed: Seed,
    pub shape: Shape,
    pub colour: String,
    pub features: Vec<Feature>,
    pub algorithm: Algorithm,
    pub stroke_width: StrokeWidth,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StrokeWidth(pub f64);

impl Default for StrokeWidth {
    fn default() -> Self {
        StrokeWidth(4.0)
    }
}

pub struct Svg(pub String);

impl Parameters {
    fn create_maze<M: Maze>(&self, template: M) -> (M, Solution<M::Idx>) {
        let mut maze = self.algorithm.execute(template);
        let solution = maze.make_solution();
        (maze, solution)
    }

    fn render<M: Maze, R: MazeRenderer<M>>(&self, mut renderer: R) -> Svg {
        for i in self.features.iter().sorted() {
            Into::<DrawingInstructions>::into(*i).run(&mut renderer)
        }
        renderer.paint(WebColour::from_string(&self.colour).unwrap());
        renderer.render()
    }

    pub fn execute(&self) -> Svg {
        fastrand::seed(self.seed);
        match self.shape {
            Shape::Rectilinear(x, y) => {
                let (maze, solution) = self.create_maze(RectilinearMaze::new((x, y)));
                self.render(RectilinearRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width / 2.0,
                    40.0,
                ))
            }
            Shape::Theta(size) => {
                let (maze, solution) = self.create_maze(RingMaze::new(size, 8));
                self.render(RingMazeRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width,
                    40.0,
                ))
            }
            Shape::Sigma(size) => {
                let (maze, solution) = self.create_maze(SigmaMaze::new(size));
                self.render(SigmaMazeRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width * 0.75,
                    40.0,
                ))
            }
        }
    }
}

impl Parameters2 {
    fn create_maze<M: Maze>(&self, template: M) -> (M, Solution<M::Idx>) {
        let mut maze = self.algorithm.execute(template);
        let solution = maze.make_solution();
        (maze, solution)
    }

    fn render<M: Maze, R: MazeRenderer<M>>(&self, mut renderer: R) -> Svg {
        for i in self.features.iter().sorted() {
            Into::<DrawingInstructions>::into(*i).run(&mut renderer)
        }
        renderer.paint(WebColour::from_string(&self.colour).unwrap());
        renderer.render()
    }

    pub fn execute(&self) -> Svg {
        fastrand::seed(self.seed.0);
        match self.shape {
            Shape::Rectilinear(x, y) => {
                let (maze, solution) = self.create_maze(RectilinearMaze::new((x, y)));
                self.render(RectilinearRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width.0 / 2.0,
                    40.0,
                ))
            }
            Shape::Theta(size) => {
                let (maze, solution) = self.create_maze(RingMaze::new(size, 8));
                self.render(RingMazeRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width.0,
                    40.0,
                ))
            }
            Shape::Sigma(size) => {
                let (maze, solution) = self.create_maze(SigmaMaze::new(size));
                self.render(SigmaMazeRenderer::new(
                    &maze,
                    &solution,
                    self.stroke_width.0 * 0.75,
                    40.0,
                ))
            }
        }
    }
}
