#![allow(mixed_script_confusables)]
pub mod metadata;
pub mod pdf;

use amazegen::maze::feature::{Configuration, Svg};
use metadata::Metadata;
use pdf::PdfWriter;
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
        let (maze, new_seed) = configuration.execute_for_svg();
        let metadata = Metadata::from_configuration(&configuration, url.clone());
        let maze_with_metadata = metadata.metadata_to_render(maze, &font_name.clone());
        let svg = Svg {
            content: maze_with_metadata.document.to_string(),
            dimensions: maze_with_metadata.dimensions,
        };
        configuration.seed = new_seed;
        pdf.append_maze(&svg);
    }
    pdf.write_to_memory()
}
