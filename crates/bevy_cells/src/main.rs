use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    // Bevy plugins
    app.add_plugins(DefaultPlugins);

    // Add our plugin
    app.add_plugins(cell_engine::CellEnginePlugin);

    app.run();
}
