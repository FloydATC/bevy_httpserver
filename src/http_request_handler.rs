
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


#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    fn test_handler_ok(_world: &mut World, _request: &Request<Bytes>) -> Result<Response<Bytes>, StatusCode> {
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Bytes::from_static(b""))
            .unwrap();

        return Ok(response);
    }

    fn test_handler_error(_world: &mut World, _request: &Request<Bytes>) -> Result<Response<Bytes>, StatusCode> {
        let response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Bytes::from_static(b""))
            .unwrap();

        return Ok(response);
    }

    #[test]
    fn new_1() {
        let _handler: HttpRequestHandler = HttpRequestHandler::new("/", test_handler_ok);
        assert!(true);
    }

    #[test]
    fn new_2() {
        let _handler: HttpRequestHandler = HttpRequestHandler::new("/", test_handler_error);
        assert!(true);
    }

    #[test]
    fn handle_root() {
        let handler: HttpRequestHandler = HttpRequestHandler::new("/", test_handler_ok);
        let request = Request::builder()
            .uri("/")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

    #[test]
    fn handle_not_found() {
        let handler: HttpRequestHandler = HttpRequestHandler::new("/", test_handler_ok);
        let request = Request::builder()
            .uri("/missing")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { assert_eq!(status, StatusCode::NOT_FOUND); }
            Ok(_) => { panic!("handler should have returned 404 Not Found"); }
        }
    }

    #[test]
    fn handle_hierarchy_1() {
        let handler: HttpRequestHandler = 
            HttpRequestHandler::new("/", test_handler_ok)
                .add_child(
                    HttpRequestHandler::new("foo", test_handler_error)
                )
                .add_child(
                    HttpRequestHandler::new("bar", test_handler_error)
                );
        let request = Request::builder()
            .uri("/")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

    #[test]
    fn handle_hierarchy_2() {
        let handler: HttpRequestHandler = 
            HttpRequestHandler::new("/", test_handler_error)
                .add_child(
                    HttpRequestHandler::new("foo", test_handler_ok)
                )
                .add_child(
                    HttpRequestHandler::new("bar", test_handler_error)
                );
        let request = Request::builder()
            .uri("/foo")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

    #[test]
    fn handle_hierarchy_3() {
        let handler: HttpRequestHandler = 
            HttpRequestHandler::new("/", test_handler_error)
                .add_child(
                    HttpRequestHandler::new("foo", test_handler_error)
                )
                .add_child(
                    HttpRequestHandler::new("bar", test_handler_ok)
                );
        let request = Request::builder()
            .uri("/bar")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

    #[test]
    fn handle_hierarchy_4() {
        let handler: HttpRequestHandler = 
            HttpRequestHandler::new("/", test_handler_error)
                .add_child(
                    HttpRequestHandler::new("foo", test_handler_ok)
                        .add_child(
                            HttpRequestHandler::new("bar", test_handler_error)
                        )
                );
        let request = Request::builder()
            .uri("/foo")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

    #[test]
    fn handle_hierarchy_5() {
        let handler: HttpRequestHandler = 
            HttpRequestHandler::new("/", test_handler_error)
                .add_child(
                    HttpRequestHandler::new("foo", test_handler_error)
                        .add_child(
                            HttpRequestHandler::new("bar", test_handler_ok)
                        )
                );
        let request = Request::builder()
            .uri("/foo/bar")
            .body(Bytes::from_static(b""))
            .unwrap();
        let mut world = World::new();

        match handler.handle(&mut world, "/", &request) {
            Err(status) => { panic!("handler returned {:?} {:?}", status.as_str(), status.canonical_reason()); }
            Ok(_) => { assert!(true) }
        }
    }

}
