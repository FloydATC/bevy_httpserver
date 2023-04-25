
/*
An HttpConnectionTask is instantiated with...
    1. an async Task handle from Bevy
    2. an Arc<Mutex<Option<Request<Bytes> for RECEIVING requests (really just a 1 item queue)
    3. an Arc<Mutex<Option<Response<Bytes> for SENDING responses (really just a 1 item queue)

This is a Bevy component facing Bevy, serving two purposes:
    1. in http_systems::http_connection_status, track the status of .get_mut_task(),
       removing tasks that have finished (connection closed or an error occurred)
    2. in http_systems::http_request_responder, use .take_request)() and .set_response()
       to serve requests.

See also: HttpConnectionServer


*/
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::tasks::Task;

use vebb::*;

#[derive(Component)]
pub struct HttpConnectionTask {
    task: Task<Result<(),String>>,
    request: Arc<Mutex<Option<Request<Bytes>>>>,
    response: Arc<Mutex<Option<Response<Bytes>>>>,
}


impl HttpConnectionTask {

    pub fn new(
        task: Task<Result<(),String>>,
        request: Arc<Mutex<Option<Request<Bytes>>>>,
        response: Arc<Mutex<Option<Response<Bytes>>>>,
    ) -> Self {
        HttpConnectionTask { 
            task, 
            request,
            response, 
        }
    }

    pub fn get_mut_task(&mut self) -> &mut Task<Result<(),String>> {
        return &mut self.task;
    }

    pub fn set_response(&mut self, response: Option<Response<Bytes>>) {
        *self.response.lock().unwrap() = response;
    }

    pub fn has_request(&self) -> bool {
        return self.request.lock().unwrap().is_some();
    }

    pub fn take_request(&mut self) -> Request<Bytes> {
        if let Some(request) = self.request.lock().unwrap().take() {
            return request;
        } else {
            panic!("can not take_request() because request is None; use has_request() first");
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let _conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        assert!(true);
    }

    #[test]
    fn get_mut_task() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let mut conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        let _: &mut Task<Result<(), String>> = conntask.get_mut_task();
    }

    #[test]
    fn has_request_none() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        assert_eq!(conntask.has_request(), false);
    }

    #[test]
    fn has_request_some() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let request = Request::builder().body(Bytes::from_static(b"")).unwrap();
        let conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(Some(request))),
            Arc::new(Mutex::new(None)),
        );
        assert_eq!(conntask.has_request(), true);
    }

    #[test]
    #[should_panic]
    fn take_request_none() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let mut conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
        );
        let _ = conntask.take_request();
    }

    #[test]
    fn take_request_some() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let request = Request::builder().body(Bytes::from_static(b"")).unwrap();
        let arc_req = Arc::new(Mutex::new(Some(request)));
        let mut conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            arc_req.clone(),
            Arc::new(Mutex::new(None)),
        );
        let _request: Request<Bytes> = conntask.take_request();
    }

    #[test]
    fn request_empty_after_take() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let request = Request::builder().body(Bytes::from_static(b"")).unwrap();
        let arc_req = Arc::new(Mutex::new(Some(request)));
        let mut conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            arc_req.clone(),
            Arc::new(Mutex::new(None)),
        );
        let _request: Request<Bytes> = conntask.take_request();
        assert_eq!(arc_req.lock().unwrap().is_some(), false);
        assert_eq!(conntask.has_request(), false);
    }

    #[test]
    fn set_response() {
        let pool = bevy::tasks::AsyncComputeTaskPool::init(|| bevy::tasks::TaskPool::new());
        let task: Task<Result<(),String>> = pool.spawn(async move { return Ok(()); });
        let response = Response::builder().status(StatusCode::OK).body(Bytes::from_static(b"")).unwrap();
        let arc_res = Arc::new(Mutex::new(None));
        let mut conntask: HttpConnectionTask = HttpConnectionTask::new(
            task,
            Arc::new(Mutex::new(None)),
            arc_res.clone(),
        );
        conntask.set_response(Some(response));
        assert_eq!(arc_res.lock().unwrap().is_some(), true);
    }

}
