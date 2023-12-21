use std::net::SocketAddr;

/// The state machine sends commands to the service.
pub enum MioCmd {
    /// Bind a new listener to a new socket on the interface
    /// that previously reported by an `InterfaceAppear` event with the ip.
    ListenOn(SocketAddr),
    /// Accept an incoming connection that previously reported
    /// by an `IncomingConnectionIsReady` event with the listener address.
    Accept(SocketAddr),
    /// Refuse to connect the incoming connection previously reported.
    Refuse(SocketAddr),
    /// Create a new outgoing connection to the socket.
    Connect(SocketAddr),
    /// Receive some data from the connection in the buffer.
    Recv(SocketAddr, Box<[u8]>),
    /// Send the data in the connection.
    Send(SocketAddr, Box<[u8]>),
    /// Disconnect the remote peer.
    Disconnect(SocketAddr),
}

pub trait P2pMioService: redux::Service {
    fn send_mio_cmd(&self, cmd: MioCmd);
}
