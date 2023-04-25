
use std::io::Write;


pub struct HttpClientConnection {
    stream: std::net::TcpStream,
    peer: std::net::SocketAddr,
    reader: std::io::BufReader<std::net::TcpStream>,
    writer: std::io::BufWriter<std::net::TcpStream>,
}


impl HttpClientConnection {

    pub fn new(stream: std::net::TcpStream, peer: std::net::SocketAddr) -> Self {
        let reader = std::io::BufReader::new(stream.try_clone().unwrap());
        let writer = std::io::BufWriter::new(stream.try_clone().unwrap());
        HttpClientConnection {
            stream,
            peer,
            reader,
            writer,
        }
    }

    pub fn peer(&self) -> &std::net::SocketAddr {
        return &self.peer;
    }

    pub fn reader(&mut self) -> &mut std::io::BufReader<std::net::TcpStream> {
        return &mut self.reader;
    }

    pub fn writer(&mut self) -> &mut std::io::BufWriter<std::net::TcpStream> {
        return &mut self.writer;
    }

    pub fn close(&mut self) -> Result<(), std::io::Error>{
        self.writer.flush()?;
        self.stream.shutdown(std::net::Shutdown::Both)?;
        return Ok(());
    }

}

