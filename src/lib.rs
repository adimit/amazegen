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

const STAIN_A: &str = "FFDC80";
const STAIN_B: &str = "B9327D";
const SOLUTION: &str = "8FE080";

#[wasm_bindgen]
pub fn make_svg_maze(
    x_size: usize,
    y_size: usize,
    seed: u64,
    colour: &str,
    stain: bool,
    solve: bool,
) -> String {
    let maze: RectilinearMaze = jarník(x_size, y_size, seed);
    let mut str = String::new();
    let mut instructions: Vec<DrawingInstructions> = vec![];
    if stain {
        instructions.push(DrawingInstructions::StainMaze((
            WebColour::from_string(STAIN_A).unwrap(),
            WebColour::from_string(STAIN_B).unwrap(),
        )))
    }
    instructions.push(DrawingInstructions::DrawMaze(
        WebColour::from_string(colour).unwrap(),
    ));
    if solve {
        instructions.push(DrawingInstructions::ShowSolution(
            WebColour::from_string(SOLUTION).unwrap(),
        ))
    }
    PlottersSvgStringWriter::new(&mut str, 40, 4)
        .write_maze(&maze, instructions)
        .unwrap();
    str
}

#[cfg(test)]
mod test {
    use crate::make_svg_maze;

    #[test]
    fn mkae_svg_maze_should_return_svg_when_params_are_valid() {
        let maze = make_svg_maze(10, 10, 1, "ffffff".into(), false, false);
        assert_eq!(maze.contains("<svg"), true)
    }
}

// This will end up being a bigint in js-land.
// Generating random bigints in js-land is a pain, so that's why we do it here.
#[wasm_bindgen]
pub fn generate_seed() -> u64 {
    fastrand::u64(..)
}
