#![allow(mixed_script_confusables)]

use amazegen::maze::{algorithms::jarník, interface::Maze, shape::sigma::SigmaMaze};

fn main() -> Result<(), ()> {
    let template = SigmaMaze::new(10);
    let (maze, solution) = {
        let mut maze = jarník(template);
        let solution = maze.make_solution();
        (maze, solution)
    };
    dbg!(maze);
    dbg!(solution);
    Ok(())
}
