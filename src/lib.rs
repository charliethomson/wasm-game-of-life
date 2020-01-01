mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const IDLE_TIME: u8 = 10;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn into_cells(i: Vec<Vec<u8>>) -> Vec<Cell> {
    i.iter().fold(Vec::new(), |mut acc, row| {
        acc.extend(
            row.iter()
                .map(|b| if b != &0 { Cell::Alive } else { Cell::Dead }),
        );
        acc
    })
}

#[test]
fn test_neighbors() {
    let board = Universe::with_cells(
        into_cells(vec![
            vec![0, 1, 0, 1],
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]),
        4,
        4,
    );
    assert_eq!(board.neighbors(2, 1), 3);
    assert_eq!(board.neighbors(1, 2), 1);
    assert_eq!(board.neighbors(2, 0), 4);
    assert_eq!(board.neighbors(0, 0), 1);
    assert_eq!(board.neighbors(3, 3), 0);
}

#[test]
fn test_tick() {
    let mut board = Universe::with_cells(
        into_cells(vec![
            vec![0, 1, 0, 1],
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]),
        4,
        4,
    );

    let tick_1 = into_cells(vec![
        vec![0, 0, 0, 1],
        vec![0, 0, 1, 1],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
    ]);

    let tick_2 = into_cells(vec![
        vec![0, 0, 1, 1],
        vec![0, 0, 1, 1],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
    ]);

    board.tick();
    assert_eq!(board.cells, tick_1);

    board.tick();
    assert_eq!(board.cells, tick_2);

    // tick_2 is the final state of this sim, so it should stay the same

    board.tick();
    assert_eq!(board.cells, tick_2);

    board.tick();
    assert_eq!(board.cells, tick_2);

    let mut board = Universe::with_cells(
        into_cells(vec![
            vec![0, 1, 1, 0],
            vec![0, 0, 0, 1],
            vec![0, 1, 1, 0],
            vec![0, 0, 0, 0],
        ]),
        4,
        4,
    );

    let tick_1 = into_cells(vec![
        vec![0, 0, 1, 0],
        vec![0, 0, 0, 1],
        vec![0, 0, 1, 0],
        vec![0, 0, 0, 0],
    ]);

    let tick_2 = into_cells(vec![
        vec![0, 0, 0, 0],
        vec![0, 0, 1, 1],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
    ]);

    let tick_final = into_cells(vec![
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
    ]);

    board.tick();

    assert_eq!(board.cells, tick_1);

    board.tick();
    assert_eq!(board.cells, tick_2);

    board.tick();
    assert_eq!(board.cells, tick_final);

    // end state

    board.tick();
    assert_eq!(board.cells, tick_final);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
    Idle = 2,
}

#[wasm_bindgen]
pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    dead_times: Vec<u8>,
}
impl Universe {
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn neighbors(&self, x: usize, y: usize) -> u8 {
        (x.checked_sub(1).unwrap_or(0)..=x.checked_add(1).unwrap_or(std::usize::MAX))
            .fold(Vec::new(), |mut acc: Vec<(usize, usize)>, ix: usize| {
                acc.extend(
                    (y.checked_sub(1).unwrap_or(0)..=y.checked_add(1).unwrap_or(std::usize::MAX))
                        .map(|iy| (ix, iy)),
                );
                acc
            })
            .iter()
            .cloned()
            .filter(|ptr| ptr != &(x, y))
            .fold(0u8, |acc, ptr| {
                let (ix, iy) = ptr;
                if let Some(Cell::Alive) = self.cells.get(self.index(ix, iy)) {
                    acc + 1
                } else {
                    acc
                }
            })
    }
}

impl Universe {
    pub fn with_cells(cells: Vec<Cell>, height: usize, width: usize) -> Self {
        Universe {
            width,
            height,
            cells,
            dead_times: (0..(width * height)).map(|_| 0).collect(),
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new(/* width: usize, height: usize */) -> Self {
        // /*
        let width = 64;
        let height = 64;
        // */

        utils::set_panic_hook();
        // let cells = (0..(width * height))
        //     .map(|_| Cell::Dead).collect();
        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Idle
                }
            })
            .collect();
        Universe {
            width,
            height,
            cells,
            dead_times: (0..(width * height)).map(|_| 0).collect(),
        }
    }

    pub fn tick(&mut self) {
        let mut new_board = self.cells.clone();

        for iy in 0..self.height {
            for ix in 0..self.width {
                let idx = self.index(ix, iy);
                // TODO: clean this, fix the brackets
                new_board[idx] = match (self.cells[idx], self.neighbors(ix, iy), self.dead_times[idx]) {
                    (Cell::Alive, n, _) if n < 2 => Cell::Dead,
                    (Cell::Alive, n, _) if n < 4 => Cell::Alive,
                    (Cell::Alive, n, _) if n > 3 => Cell::Dead,
                    (Cell::Dead, 3, _) => Cell::Alive,
                    (Cell::Idle, 3, _) => Cell::Alive,
                    (Cell::Dead, _, n) if n >= IDLE_TIME => Cell::Idle,
                    (Cell::Dead, _, _) => { self.dead_times[idx] += 1; Cell::Dead },
                    (o, _, _) => o,
                };
            }
        }

        self.cells = new_board;
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
        self.clear();        
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.clear();        
    }
    
    pub fn height(&self) -> usize {
        self.height
    }
    
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn clear(&mut self) {
        self.cells = (0..(self.width * self.height))
            .map(|_| Cell::Dead).collect();
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn push_cell(&mut self, x: usize, y: usize) {
        let idx = self.index(x, y);
        self.cells[idx] = Cell::Alive;
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
