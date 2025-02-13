use bevy::prelude::*;

/// Bevy [`Resource`] to keep track of the stats of the world
#[derive(Resource, Debug, Clone)]
pub struct Stats {
    pub spawned_particles: usize,
    pub existing_particles: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            spawned_particles: 0,
            existing_particles: 0,
        }
    }
}
