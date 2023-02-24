mod maze;
use config::Configuration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub mod config {
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
                Algorithm::GrowingTree => GrowingTreeGenerator::new(extents, *seed).generate(),
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
}

#[wasm_bindgen]
pub fn generate_maze(js: JsValue) -> String {
    let configuration: Configuration = serde_wasm_bindgen::from_value(js).unwrap();
    configuration.execute().0
}

//#[wasm_bindgen]
//pub fn test_config() -> JsValue {
//    let configuration = Configuration {
//        seed: 1,
//        shape: Shape::Rectilinear(11, 12),
//        colour: "#FF00FF".into(),
//        features: vec![Feature::Stain],
//        algorithm: Algorithm::Kruskal,
//    };
//    serde_wasm_bindgen::to_value(&configuration).unwrap()
//}

#[cfg(test)]
mod test {
    use crate::config::Configuration;

    #[test]
    fn mkae_svg_maze_should_return_svg_when_params_are_valid() {
        let svg = Configuration {
            algorithm: crate::config::Algorithm::GrowingTree,
            colour: "000000".into(),
            features: vec![],
            seed: 1,
            shape: crate::config::Shape::Rectilinear(10, 10),
        }
        .execute();
        assert_eq!(svg.0.contains("<svg"), true)
    }
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    fastrand::u64(..)
}
