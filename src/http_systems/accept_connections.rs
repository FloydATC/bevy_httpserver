
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::HttpClientAddress;
use crate::HttpClientConnection;
use crate::HttpConnectionServer;
use crate::HttpConnectionTask;
use crate::HttpServerResource;


pub fn http_accept_connections(
    server: Res<HttpServerResource>,
    mut commands: Commands,
) {
    loop {
        match server.listener().accept() {
            Err(os_error) => {
                // WouldBlock means no connections waiting; come back later
                if os_error.kind() == std::io::ErrorKind::WouldBlock { break; }
                // Any other error means something went wrong
                panic!("accept() on http listener returned {}", os_error);
            }
            Ok((stream, peer)) => {
                info!("{:?} connected", peer);    
                stream.set_nonblocking(false).expect("can't set non_blocking = false");
                let request = Arc::new(Mutex::new(None));
                let response = Arc::new(Mutex::new(None));
                let mut connserv = HttpConnectionServer::new(
                    HttpClientConnection::new(stream, peer),
                    request.clone(),
                    response.clone(),
                );
            
                let pool = AsyncComputeTaskPool::get();

                let task = pool.spawn(async move {
                    return connserv.run();
                });

                commands
                    .spawn(HttpConnectionTask::new(task, request, response))
                    .insert(HttpClientAddress(peer));
            }
        }
    }
}
