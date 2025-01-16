use clap::{command, Arg, ArgAction, ArgMatches};

use crossterm::{
    event::{poll, read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io::stdout,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

const FRAME_DURATION: Duration = Duration::from_millis(70);

use crate::snake_game::{SnakeGame, WalkieTalkie};

pub fn generate_commandline_args() -> ArgMatches {
    let matches = command!()
        .arg(
            Arg::new("Single-Threaded")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Running snake in single threaded mode. Runs multi threaded by default."),
        )
        .get_matches();
    matches
}

pub fn run_game(matches: &ArgMatches) {
    match matches.get_one::<bool>("Single-Threaded") {
        Some(v) => {
            if *v {
                single_threaded();
            } else {
                multi_threaded();
            }
        }
        None => println!("how did we get here"),
    }
}

fn single_threaded() {
    enable_raw_mode().unwrap();
    let mut local_stdout = stdout();

    let mut snake_game = SnakeGame::new();

    let frame_duration = Duration::from_millis(70);
    let mut last_update = Instant::now();

    while snake_game.quit != true {
        if poll(Duration::from_millis(1)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => {
                    let player_move = &snake_game.key_stroke_move(event);
                    match player_move {
                        Ok(_) => {}
                        Err(_) => {
                            snake_game.quit = true;
                        }
                    }
                }
                _ => {}
            };
        };

        if last_update.elapsed() >= frame_duration {
            snake_game.take_step();
            snake_game.render(&mut local_stdout);
            last_update = Instant::now();
        }
        std::thread::sleep(FRAME_DURATION);
    }
}

fn multi_threaded() {
    let snake_game = SnakeGame::new();

    let game_lock = Arc::new(Mutex::new(snake_game));

    let render_walkie_talkie = WalkieTalkie::new();

    enable_raw_mode().unwrap();

    let _render_walkie = render_walkie_talkie.pair1;
    let render_lock = game_lock.clone();

    let _render_thread = thread::spawn(move || -> std::result::Result<(), ()> {
        let talkie = render_walkie_talkie.pair2;
        let mut stdout = stdout();
        let mut i = 0;
        while talkie.1.recv().unwrap() == 0 {
            {
                let mut game = render_lock.lock().unwrap();

                game.render(&mut stdout);
            }
            println!("Rendered frame {}", i);
            i += 1;
        }
        Ok(())
    });

    let input_walkie_talkie = WalkieTalkie::new();

    let _input_walkie = input_walkie_talkie.pair1;
    let input_lock = game_lock.clone();

    let _input_thread = thread::spawn(move || -> std::result::Result<(), ()> {
        let local_lock = input_lock.clone();
        let talkie = input_walkie_talkie.pair2;
        while talkie.1.recv().unwrap() == 0 {
            {
                if match poll(Duration::from_millis(1)) {
                    Ok(it) => it,
                    Err(_) => false,
                } {
                    let mut game = local_lock.lock().unwrap();
                    if let Ok(Event::Key(event)) = read() {
                        let asd = game.key_stroke_move(event);
                        match asd {
                            Ok(_) => {}
                            Err(_) => game.quit = true,
                        }
                    }
                    game.take_step();
                } else {
                    let mut game = local_lock.lock().unwrap();
                    game.take_step();
                }
            }
            thread::sleep(FRAME_DURATION);
        }
        Ok(())
    });

    loop {
        {
            let game_lock = game_lock.lock().unwrap();

            if game_lock.quit {
                _input_walkie.0.send(1).unwrap();
                _render_walkie.0.send(1).unwrap();
                break;
            }
        }

        _input_walkie.0.send(0).unwrap();
        _render_walkie.0.send(0).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }

    disable_raw_mode().unwrap();

    _input_thread.join().unwrap().unwrap();
    _render_thread.join().unwrap().unwrap();
}
