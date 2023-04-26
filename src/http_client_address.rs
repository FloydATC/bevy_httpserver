
use std::net::SocketAddr;

use bevy::prelude::*;


#[derive(Component)]
pub struct HttpClientAddress(pub SocketAddr);
