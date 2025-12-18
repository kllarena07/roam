mod app;
mod player;

use std::{io, sync::mpsc, thread};

use crate::player::Player;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<app::Event>();
    let (own_tx, own_rx) = mpsc::channel::<app::Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        app::run_background_connection(tx_to_background_progress_events, own_rx);
    });

    let players: Vec<Player> = vec![Player { x: 0, y: 0 }];

    let mut app = app::App {
        exit: false,
        players,
        own_player: Player { x: 0, y: 0 },
    };

    // App runs on the main thread.
    let app_result = app.run(&mut terminal, event_rx, own_tx);

    // Note: If your threads need clean-up (i.e. the computation thread),
    // you should communicatie to them that the app wants to shut down.
    // This is not required here, as our threads don't use resources.
    ratatui::restore();
    app_result
}

fn handle_input_events(tx: mpsc::Sender<app::Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(app::Event::Input(key_event)).unwrap()
            }
            _ => {}
        }
    }
}
