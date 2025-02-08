#![allow(mixed_script_confusables)]
pub mod maze;

use maze::feature::{Algorithm, Configuration, Feature, Shape};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn generate_maze(js: JsValue) -> String {
    let configuration: Configuration = serde_wasm_bindgen::from_value(js).unwrap();
    configuration.run().svg
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebResponse {
    svg: String,
    hash: String,
}

#[wasm_bindgen]
pub fn run_configuration(js: JsValue) -> JsValue {
    let configuration: Configuration = serde_wasm_bindgen::from_value(js).unwrap_or_else(|err| {
        log(&format!(
            "Error parsing configuration. Using default. {:?}",
            err
        ));
        Configuration {
            algorithm: Algorithm::GrowingTree,
            colour: "#FFFFFF".into(),
            features: vec![],
            seed: generate_seed(),
            shape: Shape::Rectilinear(10, 10),
            stroke_width: 8.0,
        }
    });

    configuration
        .run()
        .serialize(&Serializer::new())
        .unwrap_or_else(|err| {
            log(&format!("Error while writing response: {:?}", err));
            JsValue::NULL
        })
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
        .run()
        .svg;
        assert!(svg.contains("<svg"))
    }
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    fastrand::u64(..)
}
