use std::iter::once;

use crate::maze::generator::growing_tree::GrowingTreeGenerator;
use crate::maze::generator::kruskal::Kruskal;
use crate::maze::generator::MazeGenerator;
use crate::maze::paint::*;
use crate::maze::regular::RectilinearMaze;
use itertools::Itertools;
const STAIN_A: &str = "FFDC80";
const STAIN_B: &str = "B9327D";
const SOLUTION: &str = "8FE080";

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Shape {
    Rectilinear(usize, usize),
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
    fn generate(&self, shape: &Shape, seed: &u64) -> RectilinearMaze {
        let extents = match shape {
            Shape::Rectilinear(x, y) => (*x, *y),
        };
        match self {
            Algorithm::Kruskal => Kruskal::new(extents, *seed).generate(),
            Algorithm::GrowingTree => {
                GrowingTreeGenerator::<(usize, usize)>::new(extents, *seed).generate()
            }
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
}

pub struct SVG(pub String);

impl Configuration {
    pub fn execute(&self) -> SVG {
        let mut str = String::new();
        PlottersSvgStringWriter::new(&mut str, 40, 4)
            .write_maze(
                &self.algorithm.generate(&self.shape, &self.seed),
                self.features
                    .iter()
                    .map(|f| Into::<DrawingInstructions>::into(*f))
                    .sorted()
                    .merge(once(DrawingInstructions::DrawMaze(
                        WebColour::from_string(&self.colour).unwrap(),
                    )))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        SVG(str)
    }
}
