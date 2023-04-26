
use bevy::prelude::*;
use smol::future;

use crate::HttpConnectionTask;


pub fn http_connection_status(
    mut query: Query<(Entity, &mut HttpConnectionTask)>,
    mut commands: Commands,
) {
    // Check status of async tasks
    for (entity, mut conntask) in query.iter_mut() {
        check_conntask_status(entity, &mut conntask, &mut commands);
    }
}


// Helper function for http_connection_status()
fn check_conntask_status(
    task_entity: Entity,
    handle: &mut HttpConnectionTask,
    commands: &mut Commands,
) {
    if let Some(result) = future::block_on(future::poll_once(handle.get_mut_task())) {
        match result {
            Ok(_) => {}
            Err(msg) => warn!("HttpConnectionTask crashed: {}", msg),
        }
        commands
            .entity(task_entity)
            .despawn();
    }
}


