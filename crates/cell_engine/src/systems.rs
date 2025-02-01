use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn update(time: Res<Time>) {
    println!("Time: {}", time.elapsed_secs());
}
