#![allow(mixed_script_confusables)]
pub mod configuration;
pub mod maze;

use configuration::{Seed, UrlParameters};
use maze::feature::{Algorithm, Feature, Parameters, Parameters2, Shape, StrokeWidth};
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
    let configuration: Parameters = serde_wasm_bindgen::from_value(js).unwrap_or_else(|err| {
        log(&format!("Error: {:?}", err));
        Parameters {
            algorithm: Algorithm::GrowingTree,
            colour: "#000000".into(),
            features: vec![],
            seed: generate_seed(),
            shape: Shape::Theta(11),
            stroke_width: 8.0,
        }
    });
    configuration.execute().0
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebMazeRequest {
    hash: String,
    colour: String,
    features: Vec<Feature>,
}

impl WebMazeRequest {
    pub fn to_configuration(&self) -> Parameters2 {
        let params: UrlParameters = UrlParameters::parse_from_string(&self.hash);
        Parameters2 {
            seed: params.seed,
            shape: params.shape,
            algorithm: params.algorithm,
            colour: self.colour.clone(),
            features: self.features.clone(),
            stroke_width: StrokeWidth(8.0),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebMazeResponse {
    svg: String,
    configuration: Parameters2,
}

#[wasm_bindgen]
pub fn generate_maze_from_hash(arg: JsValue) -> JsValue {
    let params = serde_wasm_bindgen::from_value(arg)
        .unwrap_or_else(|err| {
            log(&format!("Error: {:?}", err));
            WebMazeRequest {
                hash: "".into(),
                colour: "#FFFFFF".into(),
                features: vec![],
            }
        })
        .to_configuration();

    WebMazeResponse {
        svg: params.execute().0,
        configuration: params,
    }
    .serialize(&Serializer::new().serialize_large_number_types_as_bigints(true))
    .unwrap_or_else(|err| {
        log(&format!("Error: {:?}", err));
        JsValue::NULL
    })
}

#[wasm_bindgen]
pub fn test_config() -> JsValue {
    let configuration = Parameters2 {
        seed: Seed::default(),
        shape: Shape::Theta(11),
        colour: "#FF00FF".into(),
        features: vec![Feature::Stain],
        algorithm: Algorithm::Kruskal,
        stroke_width: StrokeWidth::default(),
    };
    log(&format!("Configuration: {:?}", configuration));
    let serializer = Serializer::new().serialize_large_number_types_as_bigints(true);
    configuration.serialize(&serializer).unwrap_or_else(|err| {
        log(&format!("Error: {:?}", err));
        JsValue::NULL
    })
}

#[cfg(test)]
mod test {
    use crate::maze::feature::Parameters;

    #[test]
    fn mkae_svg_maze_should_return_svg_when_params_are_valid() {
        let svg = Parameters {
            algorithm: crate::maze::feature::Algorithm::GrowingTree,
            colour: "000000".into(),
            features: vec![],
            seed: 1,
            shape: crate::maze::feature::Shape::Rectilinear(10, 10),
            stroke_width: 8.0,
        }
        .execute();
        assert!(svg.0.contains("<svg"))
    }
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    fastrand::u64(..)
}
