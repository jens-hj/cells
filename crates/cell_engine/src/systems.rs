use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_catppuccin::CatppuccinTheme;
use bevy_pointer_to_world::{PointerToWorldCamera, PointerWorldPosition};
use cell_particle::grid::Dimensions;
use cell_particle::particle::{Particle, ParticleKind};

use crate::{CellWorld, ParticleCell, View, WorldTexture};

/// Bevy [`Startup`] system to setup the environment
pub fn setup_environment(mut commands: Commands, theme: Res<CatppuccinTheme>) {
    // Camera
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(theme.flavor.base),
            ..default()
        },
        PointerToWorldCamera,
    ));

    // World
    commands.spawn(CellWorld::new(126, 70).with_fill(ParticleKind::Stone));
}

/// Bevy [`Startup`] system to setup the visualisation of the world
pub fn setup_view(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &CellWorld), Without<View>>,
) {
    for (entity, cell_world) in query.iter() {
        let Dimensions { width, height } = cell_world.grid.dimensions();

        let size = Vec2::new(
            width as f32 * cell_world.resolution as f32,
            height as f32 * cell_world.resolution as f32,
        );

        // Create the texture
        let mut texture = Image::new_fill(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255], // Initial black pixels with full alpha
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );

        texture.sampler = ImageSampler::nearest();

        // Enable texture to be modified from CPU
        texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        let canvas = images.add(texture);

        // Spawn a sprite bundle using the texture
        commands.spawn((
            Sprite {
                custom_size: Some(size),
                image: canvas,
                ..default()
            },
            // Transform::from_scale(Vec3::new(10.0, 0.0, 10.0)),
            WorldTexture,
        ));

        commands.entity(entity).insert(View);
    }
}

/// Bevy [`FixedUpdate`] system to update the grid
pub fn grid_update(_time: Res<Time<Fixed>>) {
    // println!("Time: {}", time.elapsed_secs());
}

/// Bevy [`Update`] system to update the visualisation of the world
pub fn view_update(
    mut images: ResMut<Assets<Image>>,
    cell_worlds: Query<&CellWorld>,
    sprites: Query<&Sprite, With<WorldTexture>>,
    theme: Res<CatppuccinTheme>,
) {
    for cell_world in cell_worlds.iter() {
        let Dimensions { width, height } = cell_world.grid.dimensions();

        // Get the texture handle
        if let Ok(sprite) = sprites.get_single() {
            if let Some(texture) = images.get_mut(&sprite.image) {
                // Update texture data based on grid state
                let mut pixel_data = vec![0; (width * height * 4) as usize];

                for y in 0..height {
                    for x in 0..width {
                        let index = ((y * width + x) * 4) as usize;
                        let Ok(cell) = cell_world.grid.get(x, y) else {
                            continue;
                        };

                        let color = cell.color(&theme.flavor).to_srgba();

                        pixel_data[index] = (color.red * 255.0) as u8;
                        pixel_data[index + 1] = (color.green * 255.0) as u8;
                        pixel_data[index + 2] = (color.blue * 255.0) as u8;
                        pixel_data[index + 3] = (color.alpha * 255.0) as u8;
                    }
                }

                texture.data = pixel_data;
            }
        }
    }
}

/// Bevy [`Update`] system to take input from the mouse
pub fn mouse_input(
    mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    pointer_world_position: Res<PointerWorldPosition>,
    mut cell_worlds: Query<&mut CellWorld>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        // also get the position of the mouse
        info!("Left mouse button pressed at {}", *pointer_world_position);

        // convert to grid position
        let Ok(mut cell_world) = cell_worlds.get_single_mut() else {
            return;
        };

        let mut grid_position =
            pointer_world_position.0 / cell_world.resolution as f32 * Vec2::new(1.0, -1.0);
        grid_position.x += cell_world.grid.dimensions().width as f32 / 2.0;
        grid_position.y += cell_world.grid.dimensions().height as f32 / 2.0;
        info!("Grid position: {:?}", grid_position);

        // set the cell to red
        let cell = cell_world
            .grid
            .get_mut(grid_position.x as usize, grid_position.y as usize)
            .unwrap();

        *cell = ParticleCell {
            content: Some(Particle::new(ParticleKind::Sand)),
        }
    }
}
