#![allow(mixed_script_confusables)]
pub mod maze;
pub mod pdf;

use maze::feature::{Algorithm, Configuration, Shape};
use pdf::PdfWriter;
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn generate_pdf(js: JsValue, pages: u32, baseurl: String, font: Vec<u8>) -> Vec<u8> {
    let mut configuration: Configuration = serde_wasm_bindgen::from_value(js).unwrap();
    let font = pdf::Font::new(font);
    let font_name = font.as_ref().map(|f| f.name.clone());
    let mut pdf = PdfWriter::new(font);
    let url = Some(baseurl);
    for _ in 0..pages {
        let (maze, new_seed) = configuration.execute_for_svg(&url, &font_name);
        configuration.seed = new_seed;
        pdf.append_maze(&maze);
    }
    pdf.write_to_memory()
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebResponse {
    pub svg: String,
    pub hash: String,
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
        .execute_for_web()
        .serialize(&Serializer::new())
        .unwrap_or_else(|err| {
            log(&format!("Error while writing response: {:?}", err));
            JsValue::NULL
        })
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
        .execute_for_web()
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
