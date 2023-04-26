/*
An HttpConnectionServer is instantiated with...
    1. a HttpClientConnection (contains the peer address, stream handle and read/write buffers)
    2. an Arc<Mutex<Option<Request<Bytes> for SENDING requests (really just a 1 item queue)
    3. an Arc<Mutex<Option<Response<Bytes> for RECEIVING responses (really just a 1 item queue)

When .run() is invoked, presumably inside an async task, the HttpConnectionServer will...
    1. read a request from the client (potentially a slow/blocking call)
    2. self.set_request() to place it into the shared request queue
    3. wait for a response to appear in the other shared response queue
    4. write the HTTP response to the client (potentially a slow/blocking call)
    5. loop unless connection keep-alive was not requested or there was an error

See also: HttpConnectionTask
*/

use std::sync::{Arc, Mutex};
use std::thread;

use bevy::prelude::*;

use vebb::*;

use super::HttpClientConnection;

pub struct HttpConnectionServer {
    connection: HttpClientConnection,
    request: Arc<Mutex<Option<Request<Bytes>>>>,
    response: Arc<Mutex<Option<Response<Bytes>>>>,
}


impl HttpConnectionServer {

    pub fn new(
        connection: HttpClientConnection,
        request: Arc<Mutex<Option<Request<Bytes>>>>,
        response: Arc<Mutex<Option<Response<Bytes>>>>,
    ) -> Self {
        HttpConnectionServer {  
            connection,
            request,
            response,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            // Read request from client and put it in self.request
            let summary;
            match vebb::read_request(self.connection.reader()) {
                Err(status) => {
                    return Err(format!("{}: {}", self.connection.peer(), status));
                }
                Ok(opt_request) => { 
                    match opt_request {
                        None => break, // Connection closed by peer
                        Some(request) => {
                            summary = format!("{} {}",request.method().as_str(), request.uri().to_string());
                            self.set_request(Some(request))
                        }
                    }
                }
            }

            // Wait for response to become ready
            while !self.has_response() { thread::yield_now(); }

            // Take response from self.response, send it to the client
            let response = self.take_response();

            let keep_alive = vebb::keep_alive_granted(&response);
            info!("{} {} {}", summary, response.status().as_str(), response.status().canonical_reason().unwrap());
            if let Err(os_error) = vebb::send_response(response, self.connection.writer()) {
                if os_error.kind() == std::io::ErrorKind::ConnectionAborted { break; } // Connection closed by peer
                return Err(format!("send_response returned {}", os_error));
            }

            if keep_alive == false { break; } // Close connection from our side
        }

        // Connection closed by peer
        match self.connection.close() {
            Err(os_error) => {
                if os_error.kind() == std::io::ErrorKind::ConnectionAborted { return Ok(()) }
                return Err(format!("{}", os_error))
            }
            Ok(()) => return Ok(()), 
        }
    }

    fn set_request(&mut self, request: Option<Request<Bytes>>) {
        *self.request.lock().unwrap() = request;
    }

    fn has_response(&self) -> bool {
        return self.response.lock().unwrap().is_some();
    }

    fn take_response(&mut self) -> Response<Bytes> {
        if let Some(response) = self.response.lock().unwrap().take() {
            return response;
        } else {
            panic!("can not take_response() because response is None; use has_response() first");
        }
    }

}


#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let _connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        assert!(true);
    }

    #[test]
    fn has_response_none() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        assert_eq!(connserv.has_response(), false);
    }

    #[test]
    fn has_response_some() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let response = Response::builder().status(StatusCode::OK).body(Bytes::from_static(b"")).unwrap();
        let connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(Some(response))),
        );
        assert_eq!(connserv.has_response(), true);
    }

    #[test]
    fn set_request() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let arc_req = Arc::new(Mutex::new(None));
        let mut connserv = HttpConnectionServer::new(
            client,
            arc_req.clone(),
            Arc::new(Mutex::new(None)),
        );
        let request = Request::builder()
            .method(Method::GET)
            .uri("/".parse::<Uri>().unwrap())
            .body(Bytes::from_static(b""))
            .unwrap();

        connserv.set_request(Some(request));
        assert_eq!(arc_req.lock().unwrap().is_some(), true);
    }

    #[test]
    #[should_panic]
    fn take_response_none() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let mut connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        let _response: Response<Bytes> = connserv.take_response();
    }

    #[test]
    fn take_response_some() {
        let (_server, client) = HttpClientConnection::loopback().unwrap();
        let response = Response::builder().status(StatusCode::OK).body(Bytes::from_static(b"")).unwrap();
        let mut connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(Some(response))),
        );
        assert_eq!(connserv.has_response(), true);
        let _response: Response<Bytes> = connserv.take_response();
        assert_eq!(connserv.has_response(), false);
    }

    #[test]
    fn run_client_close() {
        let (mut server, client) = HttpClientConnection::loopback().unwrap();
        server.close().expect("close failed");
        let mut connserv = HttpConnectionServer::new(
            client,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        let handle = thread::spawn(move || connserv.run());
        let _ = handle.join().expect("run() crashed");
        assert!(true)
    }

    #[test]
    fn run_keepalive_not_requested() {
        let (mut server, client) = HttpClientConnection::loopback().unwrap();
        let request = Request::builder()
            .version(Version::HTTP_11)
            .method(Method::GET)
            .uri("/foo".parse::<Uri>().unwrap())
            .header("Host", "localhost")
            .body(Bytes::from_static(b""))
            .unwrap();
        vebb::send_request(request, server.writer()).expect("send_request failed");
        let arc_req = Arc::new(Mutex::new(None));
        let arc_res = Arc::new(Mutex::new(None));
        let mut connserv = HttpConnectionServer::new(
            client,
            arc_req.clone(),
            arc_res.clone(),
        );
        thread::spawn(move || connserv.run());
        while arc_req.lock().unwrap().is_none() { thread::yield_now(); }
        let _request: Request<Bytes> = arc_req.lock().unwrap().take().unwrap();
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Bytes::from_static(b""))
            .unwrap();
        *arc_res.lock().unwrap() = Some(response);
        assert!(true)
    }

    #[test]
    fn run_keepalive_requested_but_denied() {
        let (mut server, client) = HttpClientConnection::loopback().unwrap();
        let request = Request::builder()
            .version(Version::HTTP_11)
            .method(Method::GET)
            .uri("/foo".parse::<Uri>().unwrap())
            .header("Host", "localhost")
            .header("Connection", "keep-alive")
            .body(Bytes::from_static(b""))
            .unwrap();
        vebb::send_request(request, server.writer()).expect("send_request failed");
        let arc_req = Arc::new(Mutex::new(None));
        let arc_res = Arc::new(Mutex::new(None));
        let mut connserv = HttpConnectionServer::new(
            client,
            arc_req.clone(),
            arc_res.clone(),
        );
        thread::spawn(move || connserv.run());
        while arc_req.lock().unwrap().is_none() { thread::yield_now(); }
        let _request: Request<Bytes> = arc_req.lock().unwrap().take().unwrap();
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Connection", "close")
            .body(Bytes::from_static(b""))
            .unwrap();
        *arc_res.lock().unwrap() = Some(response);
        assert!(true)
    }

    #[test]
    fn run_keepalive_requested_but_ignored() {
        let (mut server, client) = HttpClientConnection::loopback().unwrap();
        let request = Request::builder()
            .version(Version::HTTP_11)
            .method(Method::GET)
            .uri("/foo".parse::<Uri>().unwrap())
            .header("Host", "localhost")
            .header("Connection", "keep-alive")
            .body(Bytes::from_static(b""))
            .unwrap();
        vebb::send_request(request, server.writer()).expect("send_request failed");
        let arc_req = Arc::new(Mutex::new(None));
        let arc_res = Arc::new(Mutex::new(None));
        let mut connserv = HttpConnectionServer::new(
            client,
            arc_req.clone(),
            arc_res.clone(),
        );
        thread::spawn(move || connserv.run());
        while arc_req.lock().unwrap().is_none() { thread::yield_now(); }
        let _request: Request<Bytes> = arc_req.lock().unwrap().take().unwrap();
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Bytes::from_static(b""))
            .unwrap();
        *arc_res.lock().unwrap() = Some(response);
        assert!(true)
    }

}
