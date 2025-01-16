use parser::{generate_commandline_args, run_game};

mod parser;
mod snake_game;

fn main() {
    let parser = generate_commandline_args();
    run_game(&parser);
}
