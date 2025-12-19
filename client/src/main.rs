mod app;
mod player;
mod server;

use crate::player::Player;
use crate::server::app_server::AppServer;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let matches = Command::new("roam-client")
        .version("0.1.0")
        .about("A terminal-based roam game client")
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .help("Run in server mode (SSH server on port 22)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let server_mode = matches.get_flag("server");

    if server_mode {
        let mut server = AppServer::new();
        server.run().await
    } else {
        run_local().await
    }
}

async fn run_local() -> Result<(), anyhow::Error> {
    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = std::sync::mpsc::channel::<app::Event>();
    let (own_tx, own_rx) = std::sync::mpsc::channel::<app::Event>();

    let tx_to_input_events = event_tx.clone();
    tokio::spawn(async move {
        handle_input_events(tx_to_input_events).await;
    });

    let tx_to_background_progress_events = event_tx.clone();
    tokio::spawn(async move {
        tokio::task::spawn_blocking(move || {
            app::run_background_connection(tx_to_background_progress_events, own_rx);
        });
    });

    let players: Vec<Player> = vec![Player { x: 0, y: 0 }];

    let mut app = app::App {
        exit: false,
        players,
        own_player: Player { x: 0, y: 0 },
    };

    // App runs on the main thread.
    let app_result =
        tokio::task::spawn_blocking(move || app.run(&mut terminal, event_rx, own_tx)).await??;

    ratatui::restore();
    Ok(app_result)
}

async fn handle_input_events(tx: std::sync::mpsc::Sender<app::Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                let _ = tx.send(app::Event::Input(key_event));
            }
            _ => {}
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}
