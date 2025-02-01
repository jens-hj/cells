use bevy::prelude::*;

use crate::systems::*;

pub struct CellEnginePlugin;

impl Plugin for CellEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}
