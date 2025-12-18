use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
    time::Duration,
};

enum Event {
    Tick(u8),
    NewConnection(SocketAddr),
    SendResponse(SocketAddr, Vec<u8>),
}

fn main() {
    let addr = "0.0.0.0:3000";
    let socket = UdpSocket::bind(addr).unwrap();
    println!("Binding to {}", addr);

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_clone = event_tx.clone();
    thread::spawn(move || {
        loop {
            tx_clone.send(Event::Tick(1)).unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let mut server = Server {
        socket,
        connections: HashMap::new(),
    };

    server.run(event_tx, event_rx);
}

struct Server {
    socket: UdpSocket,
    connections: HashMap<SocketAddr, u8>,
}

impl Server {
    fn run(&mut self, event_tx: mpsc::Sender<Event>, event_rx: mpsc::Receiver<Event>) {
        let socket_clone = self.socket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let mut buf = [0; 10];
                match socket_clone.recv_from(&mut buf) {
                    Ok((_, addr)) => {
                        println!("Received data from {}", addr);
                        event_tx.send(Event::NewConnection(addr)).unwrap();
                    }
                    Err(e) => println!("recv function failed: {e:?}"),
                }
            }
        });

        loop {
            match event_rx.recv().unwrap() {
                Event::Tick(tick_amt) => {
                    for (_, value) in self.connections.iter_mut() {
                        if *value > 0 {
                            *value -= tick_amt;
                        }
                    }
                    self.connections.retain(|_, v| *v > 0);
                    println!("{:?}", self.connections);
                }
                Event::NewConnection(addr) => {
                    println!("New connection from {}", addr);
                    self.connections.insert(addr, 5);
                    let response = b"Hello from server\n".to_vec();
                    self.socket.send_to(&response, addr).unwrap();
                }
                Event::SendResponse(addr, data) => {
                    self.socket.send_to(&data, addr).unwrap();
                }
            }
        }
    }
}
