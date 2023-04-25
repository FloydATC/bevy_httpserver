
use bevy::prelude::*;
use vebb::*;

use super::http_path::*;

type HttpRequestHandlerFn = fn(&mut World, &Request<Bytes>) -> Result<Response<Bytes>, StatusCode>;


#[derive(Clone)]
pub struct HttpRequestHandler {
    dir_name: String,
    function: HttpRequestHandlerFn,
    children: Vec<HttpRequestHandler>,
}


impl HttpRequestHandler {

    pub fn new(dir_name: &str, function: HttpRequestHandlerFn) -> Self {
        HttpRequestHandler {
            dir_name: dir_name.to_owned(),
            function,
            children: vec![],
        }
    }


    pub fn add_child(mut self, handler: HttpRequestHandler) -> Self {
        if handler.dir_name.contains("/") {
            panic!("dir_name cannot contain {:?}", String::from("/"));
        }
        self.children.push(handler);
        return self
    }


    pub fn dir_name(&self) -> &str {
        return self.dir_name.as_str();
    }


    pub fn handle(&self, world: &mut World, path: &str, request: &Request<Bytes>) -> Result<Response<Bytes>, StatusCode> {
        let current_path = HttpPath::from(path);
        let request_path = HttpPath::from(request.uri().path());
        for child in self.children.iter() {
            let mut candidate = current_path.clone();
            candidate.push(child.dir_name());
            if request_path.starts_with(&candidate) {
                return child.handle(world, candidate.to_string().as_str(), request);
            }
        }
        if current_path != request_path { return Err(StatusCode::NOT_FOUND); }
        return (self.function)(world, request);
    }


    pub fn error_response(&self, status: StatusCode) -> Response<Bytes> {
        info!("error_response() for {} called", self.dir_name);
        let message = format!("{} {}", status.as_u16(), status.canonical_reason().unwrap());
        return Response::builder()
            .status(status)
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Connection", "close")
            .body(Bytes::from(message))
            .unwrap();
    }
    
}
