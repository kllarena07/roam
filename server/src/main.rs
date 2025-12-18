use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

enum Event {
    Tick(u8),
    NewConnection(SocketAddr),
    Broadcast(String),
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
            client_tick
                .send(Event::Broadcast(String::from("Tick\n")))
                .unwrap();
            thread::sleep(Duration::from_millis(33));
        }
    });

    let mut server = Server {
        socket,
        connections: Arc::new(Mutex::new(HashMap::new())),
    };

    server.run(event_tx, event_rx);
}

struct Server {
    socket: UdpSocket,
    connections: Arc<Mutex<HashMap<SocketAddr, u8>>>,
}

impl Server {
    fn run(&mut self, event_tx: mpsc::Sender<Event>, event_rx: mpsc::Receiver<Event>) {
        let socket_clone = self.socket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let mut buf = [0; 10];
                match socket_clone.recv_from(&mut buf) {
                    Ok((_, addr)) => {
                        event_tx.send(Event::NewConnection(addr)).unwrap();
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
                    println!("{:?}", *connections);
                }
                Event::NewConnection(addr) => {
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(addr, 5);
                }
                Event::Broadcast(message) => {
                    let b_msg = message.as_bytes();
                    let connections = self.connections.lock().unwrap();
                    for (addr, _) in connections.iter() {
                        let _ = self.socket.send_to(b_msg, addr);
                    }
                }
            }
        }
    }
}
