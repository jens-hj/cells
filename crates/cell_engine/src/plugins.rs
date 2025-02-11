use bevy::prelude::*;
use bevy_catppuccin::{CatppuccinTheme, Flavor};
use bevy_pointer_to_world::PointerToWorldPlugin;

use crate::systems::*;

pub struct CellEnginePlugin;

impl Plugin for CellEnginePlugin {
    fn build(&self, app: &mut App) {
        // Set up the theme
        let theme = CatppuccinTheme {
            flavor: Flavor::MOCHA,
        };
        app.insert_resource(theme);
        app.insert_resource(ClearColor(theme.flavor.base));

        // Insert plugins
        app.add_plugins(PointerToWorldPlugin);

        // Set up the systems
        app.add_systems(
            Startup,
            ((setup_environment, setup_view).chain(), setup_rules),
        );
        app.add_systems(FixedUpdate, grid_update);
        app.add_systems(Update, (view_update, mouse_input));

        #[cfg(feature = "debug")]
        {
            app.add_systems(Update, draw_active_cells);
        }
    }
}
