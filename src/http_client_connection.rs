
use std::net::{TcpStream, TcpListener, SocketAddr, Shutdown};
use std::io::{BufReader, BufWriter};

pub struct HttpClientConnection {
    stream: TcpStream,
    peer: SocketAddr,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}


impl HttpClientConnection {

    pub fn new(stream: TcpStream, peer: SocketAddr) -> Self {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());
        HttpClientConnection {
            stream,
            peer,
            reader,
            writer,
        }
    }


    // Convenience function for testing
    pub fn loopback() -> Result<(Self, Self), std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:0")?;

        // From viewpoint of the client
        let server_addr = listener.local_addr()?;
        let server = TcpStream::connect(server_addr)?;
        // From viewpoint of the server
        let (client, client_addr) = listener.accept()?;

        let conn1 = HttpClientConnection::new(server, server_addr);
        let conn2 = HttpClientConnection::new(client, client_addr);
        return Ok((conn1, conn2));
    }

    pub fn this(&self) -> SocketAddr {
        return self.stream.local_addr().unwrap();
    }

    pub fn peer(&self) -> SocketAddr {
        return self.peer;
    }

    pub fn reader(&mut self) -> &mut BufReader<TcpStream> {
        return &mut self.reader;
    }

    pub fn writer(&mut self) -> &mut BufWriter<TcpStream> {
        return &mut self.writer;
    }

    pub fn close(&mut self) -> Result<(), std::io::Error>{
        self.stream.shutdown(Shutdown::Both)?;
        return Ok(());
    }

}


#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use std::net::SocketAddr;
    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn loopback() {
        let (_server, _client) = HttpClientConnection::loopback().unwrap();
        assert!(true);
    }

    #[test]
    fn peer_not_null() {
        let (server, _client) = HttpClientConnection::loopback().unwrap();
        let null: SocketAddr = "0.0.0.0:0".parse().unwrap();
        assert_ne!(server.peer(), null);
    }

    #[test]
    fn this_not_null() {
        let (server, _client) = HttpClientConnection::loopback().unwrap();
        let null: SocketAddr = "0.0.0.0:0".parse().unwrap();
        assert_ne!(server.this(), null);
    }

    #[test]
    fn peer_ne_this() {
        let (server, _client) = HttpClientConnection::loopback().unwrap();
        assert_ne!(server.peer(), server.this());
    }

    #[test]
    fn server_peer_eq_client_this() {
        let (server, client) = HttpClientConnection::loopback().unwrap();
        assert_eq!(server.peer(), client.this());
    }

    #[test]
    fn client_peer_eq_server_this() {
        let (server, client) = HttpClientConnection::loopback().unwrap();
        assert_eq!(client.peer(), server.this());
    }

    #[test]
    fn read_write() {
        const READER: &[u8] = b"hello world";
        let mut writer: [u8; READER.len()] = [0; READER.len()];
        let (mut server, mut client) = HttpClientConnection::loopback().unwrap();
        client.writer().write(&READER).expect("write failed");
        client.writer().flush().expect("flush failed"); // Must flush, or reader() will block waiting for more data
        client.close().expect("close failed");
        server.reader().read_exact(&mut writer).expect("read failed");
        assert_eq!(&READER[..], &writer[..]);
    }

}
