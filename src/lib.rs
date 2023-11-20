pub mod maze;

use maze::feature::{Algorithm, Configuration, Feature, Shape};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn generate_maze(js: JsValue) -> String {
    let configuration: Configuration = serde_wasm_bindgen::from_value(js).unwrap();
    configuration.execute().0
}

#[wasm_bindgen]
pub fn test_config() -> JsValue {
    let configuration = Configuration {
        seed: 1,
        shape: Shape::Theta(11),
        colour: "#FF00FF".into(),
        features: vec![Feature::Stain],
        algorithm: Algorithm::Kruskal,
        stroke_width: 8.0,
    };
    serde_wasm_bindgen::to_value(&configuration).unwrap()
}

#[cfg(test)]
mod test {
    use crate::maze::feature::Configuration;

    #[test]
    fn mkae_svg_maze_should_return_svg_when_params_are_valid() {
        let svg = Configuration {
            algorithm: crate::maze::feature::Algorithm::GrowingTree,
            colour: "000000".into(),
            features: vec![],
            seed: 1,
            shape: crate::maze::feature::Shape::Rectilinear(10, 10),
            stroke_width: 8.0,
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
