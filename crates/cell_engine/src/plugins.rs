use bevy::prelude::*;
use bevy_catppuccin::{CatppuccinTheme, Flavor};
use bevy_pointer_to_world::PointerToWorldPlugin;

use crate::{systems::*, Tool};

#[cfg(feature = "debug")]
use crate::{DebugMenuState, Stats, ToggleDebugMenu};

pub struct CellEnginePlugin;

impl Plugin for CellEnginePlugin {
    fn build(&self, app: &mut App) {
        // Set up the theme
        let theme = CatppuccinTheme {
            flavor: Flavor::MOCHA,
        };
        app.insert_resource(theme);
        app.insert_resource(ClearColor(theme.flavor.base));

        // Set up fixed update
        app.insert_resource(Time::<Fixed>::from_hz(100.0));

        app.init_resource::<Tool>();

        // Insert plugins
        app.add_plugins(PointerToWorldPlugin);

        // Set up the systems
        app.add_systems(
            Startup,
            (
                (setup_environment, setup_view).chain(),
                setup_rules,
                setup_tool_text,
            ),
        );
        app.add_systems(FixedUpdate, (grid_update, mouse_input));
        app.add_systems(Update, (view_update, tool_switch, update_tool_text));

        #[cfg(feature = "debug")]
        {
            app.add_event::<ToggleDebugMenu>();
            app.init_resource::<DebugMenuState>();
            app.init_resource::<Stats>();
            app.add_systems(Startup, setup_particle_count_text);
            app.add_systems(
                Update,
                (
                    draw_active_cells,
                    existing_particle_count,
                    particle_count_text,
                    toggle_debug,
                    toggle_debug_menu.run_if(on_event::<ToggleDebugMenu>),
                ),
            );
        }
    }
}
