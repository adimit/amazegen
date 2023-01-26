mod maze;
use crate::maze::generator::*;
use crate::maze::paint::*;
use crate::maze::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn make_svg_maze(x_size: usize, y_size: usize, seed: u64, colour: &str) -> String {
    let maze: Maze = jarník(x_size, y_size, seed);
    let mut str = String::new();
    let c = WebColour::from_string(colour);
    PlottersSvgStringWriter::new(&mut str, 40, 4, c.unwrap())
        .write_maze(&maze)
        .unwrap();
    str
}

#[cfg(test)]
mod test {
    use crate::make_svg_maze;

    #[test]
    fn mkae_svg_maze_should_return_svg_when_params_are_valid() {
        let maze = make_svg_maze(10, 10, 1, "ffffff".into());
        assert_eq!(maze.contains("<svg"), true)
    }
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    fastrand::u64(..)
}
