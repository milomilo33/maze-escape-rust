// Milovan Milovanovic, E2-119-2022

use array2d::{Array2D, Error};
use std::fs::File;
use std::hash::Hash;
use std::io::{ self, BufRead, BufReader };
use std::process::exit;
use std::collections::{ HashSet, VecDeque };

#[derive(Clone, PartialEq, Debug)]
enum Direction {
    WEST,
    EAST,
    NORTH,
    SOUTH
}

#[derive(Clone)]
struct MazeCell {
    row_index: usize,
    col_index: usize,
    available_directions: Vec<Direction>,
    // doors: Vec<(Direction, bool)>, // direction, unlocked bool
    // key: bool,
    end_of_maze: bool
}

impl MazeCell {
    fn new () -> Self {
        Self {
            row_index: 0,
            col_index: 0,
            available_directions: Vec::new(),
            end_of_maze: false
        }
    }
}

// struct Maze {
//     table: Array2D<MazeCell>,
//     current_position: (u32, u32),
//     previous_positions: Vec<(u32, u32)>,
//     cells_with_doors: Vec<(u32, u32, Direction, bool)>,
//     cells_with_unlocked_doors: Vec<(u32, u32, Direction)>,
//     keys_initial: Vec<(u32, u32)>,
//     keys_left: Vec<(u32, u32)>,
//     num_keys_to_use: u32,
//     ends: Vec<(u32, u32)>,
// }

struct MazeState {
    current_position: (usize, usize),
    previous_positions: Vec<(usize, usize)>,
    cells_with_locked_doors: Vec<(usize, usize, Direction)>,
    keys_left: Vec<(usize, usize)>,
    num_keys_to_use: u32,
}

impl MazeState {
    fn new() -> Self {
        Self {
            current_position: (0, 0),
            previous_positions: Vec::new(),
            cells_with_locked_doors: Vec::new(),
            keys_left: Vec::new(),
            num_keys_to_use: 0
        }
    }
}

fn read_maze_from_file(filename: String) -> (Array2D<MazeCell>, MazeState) {
    let file = File::open(&filename);
    match file {
        Ok(file) => {
            let lines = io::BufReader::new(file).lines();
            let mut maze_table = Array2D::<MazeCell>::filled_with(MazeCell::new(), 6, 9);
            let mut maze_state = MazeState::new();
            // let mut iter = 0;
            let mut row_iter = 0;
            let mut col_iter = 0;
            for line in lines {
                match line {
                    Ok(line) => {
                        let line_vec: Vec<char> = line.chars().collect();
                        // let mut maze_cell = maze_table.get_mut_row_major(iter).unwrap();
                        let mut maze_cell = maze_table.get_mut(row_iter, col_iter).unwrap();

                        maze_cell.row_index = row_iter;
                        maze_cell.col_index = col_iter;

                        if line_vec[0] == '1' {
                            let direction = Direction::WEST;
                            maze_cell.available_directions.push(direction);
                        }
                        if line_vec[1] == '1' {
                            let direction = Direction::EAST;
                            maze_cell.available_directions.push(direction);
                        }
                        if line_vec[2] == '1' {
                            let direction = Direction::NORTH;
                            maze_cell.available_directions.push(direction);
                        }
                        if line_vec[3] == '1' {
                            let direction = Direction::SOUTH;
                            maze_cell.available_directions.push(direction);
                        }

                        if line_vec[5] == '1' {
                            let direction = Direction::WEST;
                            maze_state.cells_with_locked_doors.push((row_iter, col_iter, direction));
                        }
                        if line_vec[6] == '1' {
                            let direction = Direction::EAST;
                            maze_state.cells_with_locked_doors.push((row_iter, col_iter, direction));
                        }
                        if line_vec[7] == '1' {
                            let direction = Direction::NORTH;
                            maze_state.cells_with_locked_doors.push((row_iter, col_iter, direction));
                        }
                        if line_vec[8] == '1' {
                            let direction = Direction::SOUTH;
                            maze_state.cells_with_locked_doors.push((row_iter, col_iter, direction));
                        }

                        if line_vec[10] == '1' && line_vec[11] == '1' {
                            maze_state.keys_left.push((row_iter, col_iter));
                        }

                        if line_vec[12] == '1' && line_vec[13] == '1' {
                            maze_cell.end_of_maze = true;
                        }
                    },
                    Err(_) => {
                        println!("Couldn't read line");
                        exit(1);
                    }
                }

                col_iter += 1;
                if col_iter >= 9 {
                    row_iter += 1;
                    col_iter = 0;
                }
            }
            
            return (maze_table, maze_state);
        },
        Err(_) => {
            println!("Couldn't read file {}", &filename);
            exit(1);
        }
    }
    // Read the file line by line, and return an iterator of the lines of the file.
}

fn get_valid_neighbours(maze_table: &Array2D<MazeCell>, maze_state: &MazeState) -> Vec<MazeState> {
    let mut valid_neighbours: Vec<MazeState> = Vec::new();

    let current_cell = maze_table.get(maze_state.current_position.0, maze_state.current_position.1).unwrap();
    for direction in &current_cell.available_directions {
        let mut neighbour_position: (usize, usize);
        let opposite_direction: Direction;
        match *direction {
            Direction::WEST => {
                neighbour_position = (current_cell.row_index, current_cell.col_index - 1);
                opposite_direction = Direction::EAST;
            },
            Direction::EAST => { 
                neighbour_position = (current_cell.row_index, current_cell.col_index + 1);
                opposite_direction = Direction::WEST;
            },
            Direction::NORTH => { 
                neighbour_position = (current_cell.row_index - 1, current_cell.col_index);
                opposite_direction = Direction::SOUTH;
            },
            Direction::SOUTH => { 
                neighbour_position = (current_cell.row_index + 1, current_cell.col_index);
                opposite_direction = Direction::NORTH;
            },
        }

        let mut neighbour_state = MazeState::new();
        neighbour_state.current_position = neighbour_position;
        neighbour_state.cells_with_locked_doors = maze_state.cells_with_locked_doors.clone();
        neighbour_state.num_keys_to_use = maze_state.num_keys_to_use;

        // check door and unlock (from both sides) if needed
        if maze_state.cells_with_locked_doors.contains(&(maze_state.current_position.0, maze_state.current_position.1, direction.clone())) {
            // decrement num of keys and unlock door for next state
            if maze_state.num_keys_to_use > 0 {
                neighbour_state.num_keys_to_use -= 1;
                let current_cell_with_locked_door = (maze_state.current_position.0, maze_state.current_position.1, direction.clone());
                let neighbour_cell_with_locked_door = (neighbour_position.0, neighbour_position.1, opposite_direction);
                neighbour_state.cells_with_locked_doors.retain(|x| *x != current_cell_with_locked_door && *x != neighbour_cell_with_locked_door);
            }
            // no available keys, so neighbour is not valid
            else {
                continue;
            }
        }

        neighbour_state.keys_left = maze_state.keys_left.clone();

        // pick up key in neighbour cell if available
        if maze_state.keys_left.contains(&neighbour_position) {
            neighbour_state.num_keys_to_use += 1;
            neighbour_state.keys_left.retain(|x| *x != neighbour_position);
        }

        neighbour_state.previous_positions = maze_state.previous_positions.clone();
        neighbour_state.previous_positions.push(maze_state.current_position);

        valid_neighbours.push(neighbour_state);
    }

    return valid_neighbours;
}

fn solve_maze_bfs(maze_table: Array2D<MazeCell>, initial_maze_state: MazeState) {
    // cells visited while having visited[2] keys available
    let mut visited: HashSet<(usize, usize, u32)> = HashSet::new();
    visited.insert((initial_maze_state.current_position.0, initial_maze_state.current_position.1, initial_maze_state.num_keys_to_use));

    let mut bfs_queue: VecDeque<MazeState> = VecDeque::new();
    bfs_queue.push_back(initial_maze_state);

    let mut maze_end_state: Option<MazeState> = None;

    while !bfs_queue.is_empty() {
        let current_maze_state = bfs_queue.pop_front().unwrap();
        let current_maze_cell = maze_table.get(current_maze_state.current_position.0, current_maze_state.current_position.1).unwrap();

        // found end of maze
        if current_maze_cell.end_of_maze {
            maze_end_state = Some(current_maze_state);
            break;
        }

        for neighbour_state in get_valid_neighbours(&maze_table, &current_maze_state) {
            // ignore neighbour if it has been visited with the same number of keys available
            if !visited.contains(&(neighbour_state.current_position.0, neighbour_state.current_position.1, neighbour_state.num_keys_to_use)) {
                visited.insert((neighbour_state.current_position.0, neighbour_state.current_position.1, neighbour_state.num_keys_to_use));
                bfs_queue.push_back(neighbour_state);
            }
        }
    }

    match maze_end_state {
        Some(maze_end_state) => {
            // form & draw solution output
            write_and_draw_solution(&maze_end_state, &maze_table)
        },
        None => return
    }
}

fn write_and_draw_solution(maze_end_state: &MazeState, maze_table: &Array2D<MazeCell>) {
    println!("(row, col) indexes of solution in order:\n");
    let mut iter = 1;
    for position in &maze_end_state.previous_positions {
        println!("{}. ({}, {})", iter, position.0, position.1);
        iter += 1
    }
    println!("{}. ({}, {})", iter, maze_end_state.current_position.0, maze_end_state.current_position.1);

    println!("\nEnd of solution.\n\nTable representation of solution (0 = untraversed; 1 = traversed):\n");
    // let mut row_iter = 0;
    // let mut col_iter = 0;
    for iterator in maze_table.rows_iter() {
        for maze_cell in iterator {
            if maze_end_state.previous_positions.contains(&(maze_cell.row_index, maze_cell.col_index)) ||
                maze_end_state.current_position == (maze_cell.row_index, maze_cell.col_index) {
                    print!("1  ");
                }
            else {
                print!("0  ");
            }
        }
        println!("\n");
    }
}

fn draw_initial_maze(initial_maze_state: &MazeState, maze_table: &Array2D<MazeCell>) {
    println!("\nTable representation of initial maze:\n");
    // let mut row_iter = 0;
    // let mut col_iter = 0;
    for iterator in maze_table.rows_iter() {
        for maze_cell in iterator {
            let mut cell_num = 0;
            if initial_maze_state.keys_left.contains(&(maze_cell.row_index, maze_cell.col_index)) {
                cell_num = 1;
            }
            if maze_cell.end_of_maze {
                cell_num = 2;
            }

            if maze_cell.available_directions.contains(&Direction::WEST) {
                if initial_maze_state.cells_with_locked_doors.contains(&(maze_cell.row_index, maze_cell.col_index, Direction::WEST)) {
                    print!("D<-");
                }
                else {
                    print!("<-");
                }
            }
            if maze_cell.available_directions.contains(&Direction::NORTH) {
                if initial_maze_state.cells_with_locked_doors.contains(&(maze_cell.row_index, maze_cell.col_index, Direction::NORTH)) {
                    print!("D↑");
                }
                else {
                    print!("↑");
                }
            }

            print!("{}", cell_num);

            if maze_cell.available_directions.contains(&Direction::SOUTH) {
                if initial_maze_state.cells_with_locked_doors.contains(&(maze_cell.row_index, maze_cell.col_index, Direction::SOUTH)) {
                    print!("↓D");
                }
                else {
                    print!("↓");
                }
            }
            if maze_cell.available_directions.contains(&Direction::EAST) {
                if initial_maze_state.cells_with_locked_doors.contains(&(maze_cell.row_index, maze_cell.col_index, Direction::EAST)) {
                    print!("->D");
                }
                else {
                    print!("->");
                }
            }

            print!("  ");
        }
        println!("\n");
    }
}

fn main() {
    let (maze_table, initial_maze_state) = read_maze_from_file(String::from("maze_def.txt"));

    // for locked in &initial_maze_state.cells_with_locked_doors {
    //     println!("({}, {}, {:?})", locked.0, locked.1, locked.2);
    // }

    draw_initial_maze(&initial_maze_state, &maze_table);

    solve_maze_bfs(maze_table, initial_maze_state);
}
