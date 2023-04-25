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
        info!("running");
        loop {
            // Read request from client and put it in self.request
            info!("read request");
            match vebb::read_request(self.connection.reader()) {
                Err(status) => {
                    return Err(format!("{}: {}", self.connection.peer(), status));
                }
                Ok(opt_request) => { 
                    match opt_request {
                        None => break, // Connection closed by peer
                        Some(request) => self.set_request(Some(request)),
                    }
                }
            }

            // Wait for response to become ready
            info!("wait for response");
            while !self.has_response() { thread::yield_now(); }

            // Take response from self.response, send it to the client
            info!("send response");
            let response = self.take_response();
            let keep_alive = vebb::keep_alive_granted(&response);
            if let Err(os_error) = vebb::send_response(response, self.connection.writer()) {
                if os_error.kind() == std::io::ErrorKind::ConnectionAborted { break; } // Connection closed by peer
                return Err(format!("send_response returned {}", os_error));
            }

            if keep_alive == false { break; } // Close connection from our side
        }

        // Connection closed by peer
        info!("stopping");
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

