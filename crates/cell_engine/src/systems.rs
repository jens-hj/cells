use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_catppuccin::CatppuccinTheme;
use bevy_pointer_to_world::{PointerToWorldCamera, PointerWorldPosition};
use cell_particle::grid::{Dimensions, Grid};
use cell_particle::particle::{Particle, ParticleKind};
use cell_particle::rule::{Input, Occupancy, Output, Rule};
use percentage::Percentage;

use crate::stats::{ExistingParticleCountText, SpawnedParticleCountText};
use crate::{CellRule, CellWorld, ParticleCell, View, WorldTexture};

#[cfg(feature = "debug")]
use crate::Stats;

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
    commands.spawn(CellWorld::new(126, 70));
}

/// Bevy [`Startup`] system to setup the rules of the world
pub fn setup_rules(mut commands: Commands) {
    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![Occupancy::OccupiedBy(ParticleKind::Sand)],
                    vec![Occupancy::Vacant],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant],
                    vec![Occupancy::OccupiedBy(ParticleKind::Sand)],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                        Occupancy::Unknown,
                    ],
                    vec![Occupancy::OccupiedBy(ParticleKind::Sand), Occupancy::Vacant],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant, Occupancy::Unknown],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::Unknown,
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                    ],
                    vec![Occupancy::Vacant, Occupancy::OccupiedBy(ParticleKind::Sand)],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant, Occupancy::Unknown],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
    });
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
pub fn grid_update(cell_rules: Query<&CellRule>, mut grid: Query<&mut CellWorld>) {
    let Ok(mut cell_world) = grid.get_single_mut() else {
        warn!("No cell world found");
        return;
    };

    let rules: Vec<_> = cell_rules.iter().map(|r| r.rule.clone()).collect();
    cell_world.update(&rules);
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
    #[cfg(feature = "debug")] mut stats: ResMut<Stats>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let Ok(mut cell_world) = cell_worlds.get_single_mut() else {
            return;
        };

        let mut grid_position =
            pointer_world_position.0 / cell_world.resolution as f32 * Vec2::new(1.0, -1.0);
        grid_position.x += cell_world.grid.dimensions().width as f32 / 2.0;
        grid_position.y += cell_world.grid.dimensions().height as f32 / 2.0;

        let x = grid_position.x as usize;
        let y = grid_position.y as usize;

        // set the cell
        if let Ok(cell) = cell_world.grid.get_mut(x, y) {
            *cell = ParticleCell {
                content: Some(Particle::new(ParticleKind::Sand)),
            };

            #[cfg(feature = "debug")]
            {
                stats.spawned_particles += 1;
            }

            // Mark the cell and its neighbors as active
            for dy in y.saturating_sub(1)..=(y + 1) {
                for dx in x.saturating_sub(1)..=(x + 1) {
                    cell_world.active_cells.mark_active(dx, dy);
                }
            }
        }
    }
}

/// Bevy [`Update`] system to draw gizmos outlining the active cells
#[cfg(feature = "debug")]
pub fn draw_active_cells(
    mut gizmos: Gizmos,
    cell_worlds: Query<&CellWorld>,
    theme: Res<CatppuccinTheme>,
) {
    for cell_world in cell_worlds.iter() {
        for (x, y) in cell_world.active_cells.cells.iter() {
            gizmos.rect_2d(
                Vec2::new(
                    (*x as f32 - cell_world.grid.dimensions().width as f32 / 2.0 + 0.5)
                        * cell_world.resolution as f32,
                    (*y as f32 - cell_world.grid.dimensions().height as f32 / 2.0 + 0.5)
                        * cell_world.resolution as f32
                        * -1.0,
                ),
                Vec2::new(cell_world.resolution as f32, cell_world.resolution as f32),
                theme.flavor.red,
            );
        }
    }
}

/// Bevy [`Startup`] system to setup the text on the screen counting the number of spawned particles
#[cfg(feature = "debug")]
pub fn setup_particle_count_text(mut commands: Commands, theme: Res<CatppuccinTheme>) {
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                column_gap: Val::Px(10.0),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::default(),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(theme.flavor.red),
                SpawnedParticleCountText,
            ));

            parent.spawn((
                Text::default(),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(theme.flavor.green),
                ExistingParticleCountText,
            ));
        });
}

/// Bevy [`Update`] system to keep track of the number of existing particles
#[cfg(feature = "debug")]
pub fn existing_particle_count(cell_worlds: Query<&CellWorld>, mut stats: ResMut<Stats>) {
    for cell_world in cell_worlds.iter() {
        stats.existing_particles = cell_world
            .grid
            .iter()
            .filter(|cell| cell.content.is_some())
            .count();
    }
}

/// Bevy [`Update`] system to place text on the screen counting the number of spawned particles
/// and the number of existing particles. This is used to make sure particles don't annihilate
/// each other.
#[cfg(feature = "debug")]
pub fn particle_count_text(
    mut spawned_particle_count: Query<
        &mut Text,
        (
            With<SpawnedParticleCountText>,
            Without<ExistingParticleCountText>,
        ),
    >,
    mut existing_particle_count: Query<
        &mut Text,
        (
            With<ExistingParticleCountText>,
            Without<SpawnedParticleCountText>,
        ),
    >,
    stats: Res<Stats>,
) {
    if let Ok(mut spawned_particle_count) = spawned_particle_count.get_single_mut() {
        spawned_particle_count.0 = format!("Spawned: {}", stats.spawned_particles);
    }

    if let Ok(mut existing_particle_count) = existing_particle_count.get_single_mut() {
        existing_particle_count.0 = format!("Existing: {}", stats.existing_particles);
    }
}
