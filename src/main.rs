#![allow(mixed_script_confusables)]

use amazegen::maze::{
    algorithms::jarník,
    interface::{Maze, MazeRenderer},
    paint::{sigma::SigmaMazeRenderer, WebColour},
    shape::sigma::SigmaMaze,
};

fn main() -> Result<(), ()> {
    let template = SigmaMaze::new(10);
    let (maze, solution) = {
        let mut maze = jarník(template);
        let solution = maze.make_solution();
        (maze, solution)
    };
    let mut renderer = SigmaMazeRenderer::new(&maze, &solution, 4.0, 40.0);
    renderer.stain((
        WebColour {
            r: 230,
            g: 0,
            b: 255,
            a: 255,
        },
        WebColour {
            r: 0,
            g: 100,
            b: 230,
            a: 255,
        },
    ));
    renderer.solve(WebColour {
        r: 100,
        g: 230,
        b: 150,
        a: 255,
    });

    renderer.paint(WebColour {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let svg = renderer.render();
    println!("{}", svg.0);
    Ok(())
}
