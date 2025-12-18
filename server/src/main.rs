use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json;

enum Event {
    Tick(u8),
    NewConnection(SocketAddr),
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
    connections: Arc<Mutex<HashMap<SocketAddr, u8>>>,
}

impl Server {
    fn run(&mut self, event_tx: mpsc::Sender<Event>, event_rx: mpsc::Receiver<Event>) {
        let event_tx_clone = event_tx.clone();
        let socket_clone = self.socket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let mut buf = [0; 10];
                match socket_clone.recv_from(&mut buf) {
                    Ok((_, addr)) => {
                        event_tx_clone.send(Event::NewConnection(addr)).unwrap();
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
                    connections.insert(addr, 5);
                }
                Event::BroadcastPlayers => {
                    let players = self.players.lock().unwrap();
                    let players_list: Vec<Player> = players.values().cloned().collect();
                    let json = serde_json::to_string(&players_list).unwrap();
                    let message = format!("PLAYERS{}\n", json);
                    let connections = self.connections.lock().unwrap();
                    for (addr, _) in connections.iter() {
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
