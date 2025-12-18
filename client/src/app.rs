use crate::player::Player;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, prelude::Buffer, prelude::Rect, widgets::Widget};
use std::{io, net::UdpSocket, sync::mpsc};

pub enum Event {
    Input(crossterm::event::KeyEvent),
    SetPlayers(Vec<Player>),
}

pub struct App {
    pub exit: bool,
    pub players: Vec<Player>,
}

pub fn run_background_connection(tx: mpsc::Sender<Event>) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = "127.0.0.1:3000";
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
    }
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::SetPlayers(players) => self.players = players,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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
                    for player in &mut self.players {
                        player.y = player.y.saturating_sub(1);
                    }
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    for player in &mut self.players {
                        player.x = player.x.saturating_sub(2);
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    for player in &mut self.players {
                        player.y = player.y.saturating_add(1);
                    }
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    for player in &mut self.players {
                        player.x = player.x.saturating_add(2);
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for player in &self.players {
            player.render(area, buf);
        }
    }
}
