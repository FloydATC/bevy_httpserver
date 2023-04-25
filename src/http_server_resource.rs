
use std::net::TcpListener;
use bevy::prelude::*;

use super::HttpRequestHandler;


#[derive(Resource)]
pub struct HttpServerResource {
    listener: TcpListener,
    root: HttpRequestHandler,
}

impl HttpServerResource {

    pub fn new(listener: TcpListener, root: HttpRequestHandler) -> Self {
        if root.dir_name() != "/" { 
            panic!("root handler dir_name must be {:?}, not {:?}", String::from("/"), root.dir_name()); 
        }
        HttpServerResource {
            listener,
            root,
        }
    }

    pub fn listener(&self) -> &TcpListener {
        return &self.listener;
    }

    pub fn root(&self) -> &HttpRequestHandler {
        return &self.root;
    }

}
