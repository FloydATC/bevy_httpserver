
/*
    Add a HTTP server listening on port 80 of all local IPv4 and IPv6 interfaces,
    with a default test request handler:

    use bevy_httpserver::HttpServerPlugin;
    App::new()
        // ... other plugins, resources and systems here ...
        .add_plugin(HttpServerPlugin::default());

    Add a HTTP server listening on port 80 of all local IPv4 and IPv6 interfaces,
    with a single request handler:

    use bevy_httpserver::HttpServerPlugin;
    App::new()
        // ... other plugins, resources and systems here ...
        .add_plugin(HttpServerPlugin::new(
            "[::]:80".parse().unwrap(),
            HttpRequestHandler::new("/", wwwroot::root)
            .add_child(HttpRequestHandler::new("foo", wwwroot::foo)
                .add_child(HttpRequestHandler::new("one", wwwroot::foo_one))
                .add_child(HttpRequestHandler::new("two", wwwroot::foo_two))
            )
            .add_child(HttpRequestHandler::new("bar", wwwroot::bar)
                .add_child(HttpRequestHandler::new("one", wwwroot::bar_one))
                .add_child(HttpRequestHandler::new("two", wwwroot::bar_two))
            )
        ));

    Add a HTTP server listening on port 80 of all local IPv4 and IPv6 interfaces,
    with a hierarchy of request handlers:

    use bevy_httpserver::HttpServerPlugin;
    App::new()
        // ... other plugins, resources and systems here ...
        .add_plugin(HttpServerPlugin::new(
            "[::]:80".parse().unwrap(),
            HttpRequestHandler::new("/", my_handlers::root)
            .add_child(HttpRequestHandler::new("foo", my_handlers::foo)
                .add_child(HttpRequestHandler::new("one", my_handlers::foo_one))
                .add_child(HttpRequestHandler::new("two", my_handlers::foo_two))
            )
            .add_child(HttpRequestHandler::new("bar", other_handlers::bar)
                .add_child(HttpRequestHandler::new("one", other_handlers::bar_one))
                .add_child(HttpRequestHandler::new("two", other_handlers::bar_two))
            )
        ));

    Every handler function must have the same signature:
    fn(&mut World, &Request<Bytes>) -> Result<Response<Bytes>, StatusCode>

    The built-in handler used by HttpServerPlugin::default() is shown below.

 */


use bevy::prelude::*;

pub use vebb::{Request, Response, StatusCode, Method, HeaderName, HeaderValue, HeaderMap, Uri, Bytes};

mod http_path;
mod http_client_address;
mod http_client_connection;
mod http_connection_server;
mod http_connection_task;
mod http_request_handler;
mod http_server_resource;
mod http_server_plugin;
mod http_systems;

pub use http_client_address::*;
pub use http_client_connection::*;
pub use http_connection_server::*;
pub use http_connection_task::*;
pub use http_request_handler::*;
pub use http_server_resource::*;
pub use http_server_plugin::*;
pub use http_systems::*;


pub fn example_handler_fn(
    _world: &mut World, 
    _request: &Request<Bytes>
) -> Result<Response<Bytes>, StatusCode> {

    /*
    // https://docs.rs/bevy/latest/bevy/ecs/system/struct.SystemState.html
    let mut system_state: bevy::ecs::system::SystemState<(
        EventWriter<MyEvent>,
        Res<Foo>,
        Query<(Entity, &mut Bar)>,
    )> = bevy::ecs::system::SystemState::new(world);

    let (writer, foo, mut query) = system_state.get_mut(world);

    for (entity, mut bar) in query.iter_mut() {
        // ...
    }

    // Or use a variant of the Commands pattern directly on the World reference:
    world.entity_mut(entity).get_mut::<Bar>()
    world.entity_mut(entity).despawn()
    */

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Bytes::from_static(b"Hello world"))
        .unwrap();

    return Ok(response);
    // or, for example return Err(StatusCode::NOT_FOUND);

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_example_handler_fn_signature() {
        let handler = HttpRequestHandler::new("/", example_handler_fn);
        assert_eq!(handler.dir_name(), "/");
    }
}
