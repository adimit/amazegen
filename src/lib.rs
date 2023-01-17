mod maze;
use maze::generator::*;
use maze::paint::*;
use maze::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_svg_maze(x_size: usize, y_size: usize, seed: u64) -> String {
    let maze: Maze = jarník(x_size, y_size, seed);
    let mut str = String::new();
    PlottersSvgStringWriter::new(&mut str, 40, 4)
        .write_maze(&maze)
        .unwrap();
    str
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
// Technically, even a u64 doesn't generate a really great amount of entropy
// for our RNG, but it's enough to generate cute mazes.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    rand::prelude::random()
}
