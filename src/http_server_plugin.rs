
use std::net::SocketAddr;

use bevy::prelude::*;
use bevy::app::App;

use super::HttpRequestHandler;
use super::HttpServerResource;

pub struct HttpServerPlugin {
    bind_address: SocketAddr,
    root: HttpRequestHandler,
}


impl HttpServerPlugin {

    pub fn new(bind_address: SocketAddr, root: HttpRequestHandler) -> Self {
        HttpServerPlugin {
            bind_address,
            root,
        }
    }

}


impl Default for HttpServerPlugin {

    fn default() -> Self {
        return HttpServerPlugin::new(
            "[::]:80".parse().unwrap(),
            HttpRequestHandler::new("/", super::example_handler_fn),
        );
    }

}


impl Plugin for HttpServerPlugin {

    // Configures the App to which this plugin is added.
    fn build(&self, app: &mut App) {

        let listener = vebb::listener(self.bind_address).unwrap();
        listener.set_nonblocking(true).expect("can't set nonblocking = true");

        let config = HttpServerResource::new(
            listener, 
            self.root.clone(),
        );

        app
            .insert_resource(config)
            .add_system(super::http_accept_connections)
            .add_system(super::http_connection_status)
            .add_system(super::http_request_responder)
        ;
    }

    // Runs after all plugins are built, but before the app runner is called. 
    // This can be useful if you have some resource that other plugins need 
    // during their build step, but after build you want to remove it and 
    // send it to another thread.
    fn setup(&self, _app: &mut App) {
    
    }

    // Configures a name for the Plugin which is primarily used for checking 
    // plugin uniqueness and debugging.
    fn name(&self) -> &str {
        "HttpServerPlugin"
    }

    // If the plugin can be meaningfully instantiated several times in an App, 
    // override this method to return false.
    fn is_unique(&self) -> bool {
        return true;
    }

}

