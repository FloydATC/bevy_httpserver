
use bevy::prelude::*;

use vebb::*;

use crate::HttpConnectionTask;
use crate::HttpServerResource;


// This system has World access, which means it can read/write any entity, component or resource
pub fn http_request_responder(
    world: &mut World,
) {
    // https://docs.rs/bevy/latest/bevy/ecs/system/struct.SystemState.html
    let mut system_state: bevy::ecs::system::SystemState<(
        Res<HttpServerResource>,
        Query<(Entity, &mut HttpConnectionTask)>,
    )> = bevy::ecs::system::SystemState::new(world);

    // Clone the server root request handler
    let (server, mut query) = system_state.get_mut(world);
    let server_root = server.root().clone();

    // For any HttpConnectionTask that has a request pending, get the request
    // This borrows &mut World only temporarily because the requests are taken, not borrowed
    let mut requests = Vec::<(Entity, Request<Bytes>)>::new();
    for (entity, mut conntask) in query.iter_mut() {
        if conntask.has_request() {
            requests.push((entity, conntask.take_request()));
        }
    }

    // Handle each request and put each response back into each HttpConnectionTask
    for (entity, request) in requests {
        let response = match server_root.handle(world, "/", &request) {
            Err(status) => server_root.error_response(status),
            Ok(mut response) => {
                finalize_response(&request, &mut response);
                response
            }
        };
        match world.entity_mut(entity).get_mut::<HttpConnectionTask>() {
            None => {} // Entity and/or HttpConnectionTask is gone, drop response
            Some(mut conntask) => { 
                conntask.set_response(Some(response)); 
            }
        }
    }
}


// Helper function for http_request_responder()
fn finalize_response(request: &Request<Bytes>, response: &mut Response<Bytes>) {
    if vebb::keep_alive_requested(request) && !vebb::keep_alive_denied(response) {
        vebb::header_if_missing(response, "Connection", "keep-alive");
        vebb::header_if_missing(response, "Keep-Alive", "timeout=30, max=1000");
    } else {
        vebb::header_if_missing(response, "Connection", "close");
    }
    let len = format!("{}", response.body().len());
    header_if_missing(response, "Content-Length", len.as_str());
    header_if_missing(response, "Content-Type", "text/html; charset=utf-8");
}

