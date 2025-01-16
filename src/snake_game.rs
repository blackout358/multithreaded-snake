use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode},
    ExecutableCommand, QueueableCommand,
};
use rand::{thread_rng, Rng};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    io::{stdout, Stdout, Write},
    sync::mpsc::{Receiver, Sender},
};

use std::sync::mpsc::channel;

const WIDTH: usize = 62;
const HEIGHT: usize = 26;

use std::collections::VecDeque;

pub struct WalkieTalkie {
    pub pair1: (Sender<u8>, Receiver<u8>),
    pub pair2: (Sender<u8>, Receiver<u8>),
}

impl WalkieTalkie {
    pub fn new() -> Self {
        let (tx1, rx1) = channel::<u8>();
        let (tx2, rx2) = channel::<u8>();

        Self {
            pair1: (tx1, rx2),
            pair2: (tx2, rx1),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub struct Snake {
    pub cells: VecDeque<(u8, u8)>,
    pub dx: i8,
    pub dy: i8,
    pub direction: Direction,
}

impl Snake {
    pub fn new() -> Self {
        Self {
            cells: VecDeque::new(),
            dx: 1,
            dy: 0,
            direction: Direction::Right,
        }
    }
}
pub struct SnakeGame {
    pub board: [[u8; WIDTH]; HEIGHT],
    _next_colour: u8,
    snake: Snake,
    pub quit: bool,
    pub highscores: BinaryHeap<Reverse<u16>>,
}

impl SnakeGame {
    pub fn new() -> Self {
        let mut board: [[u8; WIDTH]; HEIGHT] = [[0; WIDTH]; HEIGHT];
        let mut _stdout = stdout();

        for (i, row) in board.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                if i == 0 || i == HEIGHT - 1 || j == 0 || j == WIDTH - 1 {
                    *value += 1;
                }
            }
        }

        let mut rng = rand::thread_rng();
        let mut x: usize = rng.gen_range(1..board[0].len() - 1);
        let mut y: usize = rng.gen_range(1..board.len() - 1);

        board[y][x] = 3;

        let mut snake = Snake::new();
        snake.cells.push_front((y as u8, x as u8));

        x = rng.gen_range(1..board[0].len() - 1);
        y = rng.gen_range(1..board.len() - 1);

        board[y][x] = 2;

        let game = SnakeGame {
            board,
            snake,
            _next_colour: 0,
            quit: false,
            highscores: BinaryHeap::new(),
        };
        game
    }

    pub fn _display_nums(&self) -> String {
        let mut display = String::new();

        for row in &self.board {
            for col in row {
                display.push_str(&col.to_string());
            }
            display.push_str("\n");
        }
        display
    }

    pub fn render(&mut self, stdout: &mut std::io::Stdout) {
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();

        let mut cursour_y = 0;
        for (y, row) in self.board.iter().enumerate() {
            for (_, &cell) in row.iter().enumerate() {
                let char_to_render = match cell {
                    1 => "█".magenta(), // Wall
                    2 => "█".red(),     // Food
                    3 => "█".blue(),    // Snake
                    _ => " ".dark_grey(),
                };
                stdout
                    .queue(style::PrintStyledContent(char_to_render))
                    .unwrap();
            }
            stdout.queue(cursor::MoveTo(0, y as u16 + 1)).unwrap();
            cursour_y = y;
        }
        cursour_y += 1;

        Self::cursor_newline(stdout, &mut cursour_y);

        stdout
            .queue(style::Print(&format!("Score: {}", self.snake.cells.len())))
            .unwrap();

        Self::cursor_newline(stdout, &mut cursour_y);
        Self::cursor_newline(stdout, &mut cursour_y);

        stdout
            .queue(style::Print(&format!("   | Highscores")))
            .unwrap();
        Self::cursor_newline(stdout, &mut cursour_y);

        stdout
            .queue(style::Print(&format!(
                "{}+{}",
                "-".repeat(3),
                "-".repeat(12)
            )))
            .unwrap();
        Self::cursor_newline(stdout, &mut cursour_y);
        for (i, score) in self.highscores.clone().into_sorted_vec().iter().enumerate() {
            stdout
                .queue(style::Print(&format!("{: >2} | {}", i + 1, score.0)))
                .unwrap();
            Self::cursor_newline(stdout, &mut cursour_y);
        }

        stdout.flush().unwrap();
    }

    pub fn cursor_newline(stdout: &mut Stdout, cursour_y: &mut usize) {
        *cursour_y += 1;
        stdout.queue(cursor::MoveTo(0, *cursour_y as u16)).unwrap();
    }

    pub fn key_stroke_move(
        &mut self,
        event: crossterm::event::KeyEvent,
    ) -> std::result::Result<(), ()> {
        match event.code {
            crossterm::event::KeyCode::Backspace => {
                disable_raw_mode().unwrap();
                Err(())
            }
            crossterm::event::KeyCode::Left => {
                if self.snake.direction != Direction::Right {
                    self.snake.dy = 0;
                    self.snake.dx = -1;
                    self.snake.direction = Direction::Left;
                }
                Ok(())
            }
            crossterm::event::KeyCode::Right => {
                if self.snake.direction != Direction::Left {
                    self.snake.dy = 0;
                    self.snake.dx = 1;
                    self.snake.direction = Direction::Right;
                }
                Ok(())
            }
            crossterm::event::KeyCode::Up => {
                if self.snake.direction != Direction::Down {
                    self.snake.dx = 0;
                    self.snake.dy = -1;
                    self.snake.direction = Direction::Up;
                }
                Ok(())
            }
            crossterm::event::KeyCode::Down => {
                if self.snake.direction != Direction::Up {
                    self.snake.dy = 1;
                    self.snake.dx = 0;
                    self.snake.direction = Direction::Down;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn take_step(&mut self) {
        let current_pos = self.snake.cells.front().unwrap();

        let mut new_x = self.wrap_position(self.snake.dx + current_pos.1 as i8, WIDTH);
        let mut new_y = self.wrap_position(self.snake.dy + current_pos.0 as i8, HEIGHT);

        self.snake.cells.push_front((new_y, new_x));

        if self.board[new_y as usize][new_x as usize] == 2 {
            self.place_food();
        } else if self.board[new_y as usize][new_x as usize] == 3 {
            self.highscores
                .push(Reverse(self.snake.cells.len() as u16 - 1));
            self.reset_board();

            let mut rng = thread_rng();
            new_x = rng.gen_range(1..self.board[0].len() - 1) as u8;
            new_y = rng.gen_range(1..self.board.len() - 1) as u8;

            self.snake.cells.clear();
            self.snake.cells.push_front((new_y, new_x));

            println!("y:{}x:{}", new_y, new_x);

            self.place_food();
        } else {
            let ret = self.snake.cells.pop_back().unwrap();
            self.board[ret.0 as usize][ret.1 as usize] = 0;
        }

        self.board[new_y as usize][new_x as usize] = 3;
    }

    fn wrap_position(&self, pos: i8, max: usize) -> u8 {
        if pos < 1 {
            (max as i8 - 2) as u8
        } else if pos > (max - 2) as i8 {
            1
        } else {
            pos as u8
        }
    }

    pub fn reset_board(&mut self) {
        let mut board: [[u8; WIDTH]; HEIGHT] = [[0; WIDTH]; HEIGHT];

        for (i, row) in board.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                if i == 0 || i == HEIGHT - 1 || j == 0 || j == WIDTH - 1 {
                    *value += 1;
                }
            }
        }

        self.board = board;
    }

    pub fn place_food(&mut self) {
        let mut rng = rand::thread_rng();
        let mut x: usize = rng.gen_range(1..self.board.len() - 1);
        let mut y: usize = rng.gen_range(1..self.board[0].len() - 1);

        while self.board[x][y] != 0 {
            x = rng.gen_range(1..self.board.len() - 1);
            y = rng.gen_range(1..self.board[0].len() - 1);
        }

        self.board[x][y] = 2;
    }
}
