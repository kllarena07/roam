use crate::heartbeat_udpsocket::HeartbeatUDPSocket;

mod heartbeat_udpsocket;

fn main() {
    let socket = HeartbeatUDPSocket::bind("0.0.0.0:3000");
    socket.run();
}
