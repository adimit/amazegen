use crate::maze::feature::Algorithm;

use super::{interface::MazeGen, paint::theta::RingMazeSvg};

/*
fn debug_maze(maze: &RingMaze) {
    for cell in maze.cells.iter() {
        print!("({}, {}): ", cell.coordinates.row, cell.coordinates.column);
        for neighbour in cell.inaccessible_neighbours.iter() {
            print!("[{}, {}] ", neighbour.row, neighbour.column);
        }
        print!("| ");
        for neighbour in cell.accessible_neighbours.iter() {
            print!("({}, {}) ", neighbour.row, neighbour.column);
        }
        println!();
    }
}
*/

pub fn test_maze() {
    let mazegen = RingMazeSvg {
        cell_size: 40.0,
        size: 100,
        colour: "black".into(),
        stroke_width: 4.0,
    };
    let _str = mazegen.create_maze(
        fastrand::get_seed(),
        vec![
            /*
            DrawingInstructions::StainMaze((
                WebColour {
                    r: 255,
                    g: 50,
                    b: 255,
                    a: 255,
                },
                WebColour {
                    r: 50,
                    g: 120,
                    b: 255,
                    a: 255,
                },
            )),
            DrawingInstructions::ShowSolution(WebColour {
                r: 255,
                g: 128,
                b: 255,
                a: 255,
        }),
            */
        ],
        &Algorithm::GrowingTree,
    );
    // println!("{}", str);
}
