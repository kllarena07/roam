use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

enum Event {
    Tick(u32),
    NewConnection(SocketAddr),
    UpdatePlayer(SocketAddr, Player),
    BroadcastPlayers,
}

fn main() {
    let addr = "0.0.0.0:3000";
    let socket = UdpSocket::bind(addr).unwrap();
    println!("Binding to {}", addr);

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let server_tick = event_tx.clone();
    thread::spawn(move || {
        loop {
            server_tick.send(Event::Tick(1)).unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });
    let client_tick = event_tx.clone();
    thread::spawn(move || {
        loop {
            client_tick.send(Event::BroadcastPlayers).unwrap();
            thread::sleep(Duration::from_millis(33));
        }
    });

    let mut server = Server {
        socket,
        players: Arc::new(Mutex::new(HashMap::new())),
        connections: Arc::new(Mutex::new(HashMap::new())),
    };

    server.run(event_tx, event_rx);
}

struct Server {
    socket: UdpSocket,
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
    connections: Arc<Mutex<HashMap<SocketAddr, u32>>>,
}

impl Server {
    fn run(&mut self, event_tx: mpsc::Sender<Event>, event_rx: mpsc::Receiver<Event>) {
        const PLAYER_LIFETIME: u32 = 60;
        let event_tx_clone = event_tx.clone();
        let socket_clone = self.socket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let mut buf = [0; 1024];
                match socket_clone.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                            let trimmed = msg.trim();
                            if trimmed == "CONNECT" {
                                event_tx_clone.send(Event::NewConnection(addr)).unwrap();
                            } else if let Ok(player) = serde_json::from_str::<Player>(trimmed) {
                                event_tx_clone
                                    .send(Event::UpdatePlayer(addr, player))
                                    .unwrap();
                            }
                        }
                    }
                    Err(e) => println!("recv function failed: {e:?}"),
                }
            }
        });

        loop {
            match event_rx.recv().unwrap() {
                Event::Tick(tick_amt) => {
                    let mut connections = self.connections.lock().unwrap();
                    for (_, value) in connections.iter_mut() {
                        if *value > 0 {
                            *value -= tick_amt;
                        }
                    }
                    connections.retain(|_, v| *v > 0);
                    let active_addrs: std::collections::HashSet<SocketAddr> =
                        connections.keys().cloned().collect();
                    let mut players = self.players.lock().unwrap();
                    players.retain(|addr, _| active_addrs.contains(addr));
                    println!(
                        "Active connections: {:?}, Players: {:?}",
                        *connections, *players
                    );
                }
                Event::NewConnection(addr) => {
                    let player = Player { x: 0, y: 0 };
                    let mut players = self.players.lock().unwrap();
                    players.insert(addr, player);
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(addr, PLAYER_LIFETIME);
                }
                Event::UpdatePlayer(addr, player) => {
                    let mut players = self.players.lock().unwrap();
                    players.insert(addr, player);
                }
                Event::BroadcastPlayers => {
                    let players = self.players.lock().unwrap();
                    let connections = self.connections.lock().unwrap();
                    for (addr, _) in connections.iter() {
                        let others: Vec<Player> = players
                            .iter()
                            .filter(|(a, _)| *a != addr)
                            .map(|(_, p)| p.clone())
                            .collect();
                        let json = serde_json::to_string(&others).unwrap();
                        let message = format!("PLAYERS{}\n", json);
                        let _ = self.socket.send_to(message.as_bytes(), *addr);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Player {
    pub x: u16,
    pub y: u16,
}
