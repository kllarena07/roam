use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct HeartbeatUDPSocket {
    socket: UdpSocket,
    connections: Arc<Mutex<HashMap<SocketAddr, u8>>>,
}

fn handle_traffic(socket: UdpSocket, connections: Arc<Mutex<HashMap<SocketAddr, u8>>>) {
    loop {
        let mut buf = [0; 10]; // arbitrary amount of bytes
        match socket.recv_from(&mut buf) {
            Ok((_, addr)) => {
                let mut connections = connections.lock().unwrap();
                connections.insert(addr, 5);
            }
            Err(e) => println!("recv function failed: {e:?}"),
        }
    }
}

impl HeartbeatUDPSocket {
    pub fn bind(addr: &str) -> Self {
        let socket = UdpSocket::bind(addr).unwrap();

        let connections: Arc<Mutex<HashMap<SocketAddr, u8>>> = Arc::new(Mutex::new(HashMap::new()));

        Self {
            socket,
            connections,
        }
    }
    pub fn run(self) {
        let connections_clone = Arc::clone(&self.connections);
        thread::spawn(move || {
            handle_traffic(self.socket, connections_clone);
        });

        loop {
            println!("{:?}", self.connections.lock().unwrap());
            {
                let mut guard = self.connections.lock().unwrap();
                for (_, value) in guard.iter_mut() {
                    if *value > 0 {
                        *value -= 1;
                    }
                }
                guard.retain(|_, v| *v > 0);
            }

            thread::sleep(Duration::from_millis(1000));
        }
    }
}
