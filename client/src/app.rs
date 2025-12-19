use crate::player::Player;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, prelude::Buffer, prelude::Rect, widgets::Widget};
use std::{io, net::UdpSocket, sync::mpsc};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};

pub enum Event {
    Input(crossterm::event::KeyEvent),
    SetPlayers(Vec<Player>),
    OwnPosition(Player),
}

#[derive(Clone)]
pub struct App {
    pub exit: bool,
    pub players: Vec<Player>,
    pub own_player: Player,
}

pub fn run_background_connection(tx: mpsc::Sender<Event>, own_rx: mpsc::Receiver<Event>) {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let server_addr = "100.119.22.31:3000";
    if let Err(e) = socket.send_to(b"CONNECT", server_addr) {
        eprintln!("Failed to connect to server: {}", e);
        return;
    }

    loop {
        let mut buf = [0; 1024];
        if let Ok((size, _)) = socket.recv_from(&mut buf)
            && let Ok(msg) = std::str::from_utf8(&buf[..size])
            && msg.starts_with("PLAYERS")
        {
            let json_start = "PLAYERS".len();
            if let Some(json_str) = msg.get(json_start..).and_then(|s| s.strip_suffix('\n'))
                && let Ok(players) = serde_json::from_str::<Vec<Player>>(json_str)
            {
                let _ = tx.send(Event::SetPlayers(players));
            }
        }
        // Handle own position updates
        if let Ok(Event::OwnPosition(player)) = own_rx.try_recv() {
            let json = serde_json::to_string(&player).unwrap();
            let _ = socket.send_to(json.as_bytes(), server_addr);
        }
    }
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
        tx: mpsc::Sender<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => {
                    self.handle_key_event(key_event)?;
                    let _ = tx.send(Event::OwnPosition(self.own_player.clone()));
                }
                Event::SetPlayers(players) => self.players = players,
                _ => {}
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true;
                }
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Char('w') | KeyCode::Up => {
                    self.own_player.y = self.own_player.y.saturating_sub(1);
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    self.own_player.x = self.own_player.x.saturating_sub(2);
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    self.own_player.y = self.own_player.y.saturating_add(1);
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    self.own_player.x = self.own_player.x.saturating_add(2);
                }
                _ => {}
            };
        }

        Ok(())
    }
}

impl App {
    pub fn handle_event(&mut self, event: Event, tx: &UnboundedSender<Event>) {
        match event {
            Event::Input(key_event) => {
                let _ = self.handle_key_event(key_event);
                let _ = tx.send(Event::OwnPosition(self.own_player.clone()));
            }
            Event::SetPlayers(players) => self.players = players,
            _ => {}
        }
    }
}

pub async fn run_background_connection_async(tx: UnboundedSender<Event>, mut own_rx: UnboundedReceiver<Event>) {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let server_addr = "100.119.22.31:3000";
    if let Err(e) = socket.send_to(b"CONNECT", server_addr).await {
        eprintln!("Failed to connect to server: {}", e);
        return;
    }

    loop {
        let mut buf = [0; 1024];
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                if let Ok((size, _)) = result
                    && let Ok(msg) = std::str::from_utf8(&buf[..size])
                    && msg.starts_with("PLAYERS")
                {
                    let json_start = "PLAYERS".len();
                    if let Some(json_str) = msg.get(json_start..).and_then(|s| s.strip_suffix('\n'))
                        && let Ok(players) = serde_json::from_str::<Vec<Player>>(json_str)
                    {
                        let _ = tx.send(Event::SetPlayers(players));
                    }
                }
            }
            event = own_rx.recv() => {
                if let Some(Event::OwnPosition(player)) = event {
                    let json = serde_json::to_string(&player).unwrap();
                    let _ = socket.send_to(json.as_bytes(), server_addr).await;
                }
            }
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for player in &self.players {
            player.render(area, buf);
        }
        self.own_player.render(area, buf);
    }
}
