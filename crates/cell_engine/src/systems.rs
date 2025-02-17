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

use crate::{CellRule, CellWorld, ParticleCell, Tool, ToolText, View, WorldTexture};
#[cfg(feature = "debug")]
use crate::{
    DebugMenu, DebugMenuState, ExistingParticleCountText, SpawnedParticleCountText, ToggleDebugMenu,
};

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
    // Sand
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
        priority: None,
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                        Occupancy::Unknown,
                    ],
                    vec![Occupancy::OccupiedByAny, Occupancy::Vacant],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant, Occupancy::Unknown],
                    vec![
                        Occupancy::OccupiedByAny,
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: None,
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::Unknown,
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                    ],
                    vec![Occupancy::Vacant, Occupancy::OccupiedByAny],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Unknown, Occupancy::Vacant],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Sand),
                        Occupancy::OccupiedByAny,
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: None,
    });

    // Water
    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![Occupancy::OccupiedBy(ParticleKind::Water)],
                    vec![Occupancy::Vacant],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant],
                    vec![Occupancy::OccupiedBy(ParticleKind::Water)],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: Some(0),
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::Unknown,
                    ],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::Vacant,
                    ],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Vacant, Occupancy::Unknown],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: Some(1),
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::Unknown,
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                    vec![
                        Occupancy::Vacant,
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![Occupancy::Unknown, Occupancy::Vacant],
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: Some(1),
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::Vacant,
                    ],
                    vec![Occupancy::OccupiedByAny, Occupancy::OccupiedByAny],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::Vacant,
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                    vec![Occupancy::OccupiedByAny, Occupancy::OccupiedByAny],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: Some(2),
    });

    commands.spawn(CellRule {
        rule: Rule {
            input: Input {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::Vacant,
                        Occupancy::OccupiedBy(ParticleKind::Water),
                    ],
                    vec![Occupancy::OccupiedByAny, Occupancy::OccupiedByAny],
                ])
                .unwrap(),
            },
            output: vec![Output {
                grid: Grid::new(vec![
                    vec![
                        Occupancy::OccupiedBy(ParticleKind::Water),
                        Occupancy::Vacant,
                    ],
                    vec![Occupancy::OccupiedByAny, Occupancy::OccupiedByAny],
                ])
                .unwrap(),
                probability: Percentage::new(1.0),
            }],
        },
        priority: Some(2),
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

    let rules: Vec<_> = cell_rules.iter().map(|r| r.clone()).collect();
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
    tool: Res<Tool>,
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
            match *tool {
                Tool::Despawn => {
                    *cell = ParticleCell { content: None };
                }
                Tool::Spawn(particle_kind) => {
                    *cell = ParticleCell {
                        content: Some(Particle::new(particle_kind)),
                    };
                }
            }

            // Mark the cell and its neighbors as active
            for dy in y.saturating_sub(1)..=(y + 1) {
                for dx in x.saturating_sub(1)..=(x + 1) {
                    cell_world.active_cells.mark_active(dx, dy);
                }
            }

            #[cfg(feature = "debug")]
            {
                stats.spawned_particles += 1;
            }
        }
    }
}

/// Bevy [`Update`] system to switch between tools, selects tool based on number keys
pub fn tool_switch(keyboard_input: ResMut<ButtonInput<KeyCode>>, mut tool: ResMut<Tool>) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        *tool = Tool::Despawn;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        *tool = Tool::Spawn(ParticleKind::Sand);
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        *tool = Tool::Spawn(ParticleKind::Water);
    } else if keyboard_input.just_pressed(KeyCode::Digit4) {
        *tool = Tool::Spawn(ParticleKind::Stone);
    }
}

/// Bevy [`Startup`] system to setup the text to display the current tool
pub fn setup_tool_text(mut commands: Commands, theme: Res<CatppuccinTheme>) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        })
        .with_child((
            Text::default(),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(theme.flavor.lavender),
            ToolText,
        ));
}

/// Bevy [`Update`] system to update the text to display the current tool
pub fn update_tool_text(tool: Res<Tool>, mut tool_text: Query<&mut Text, With<ToolText>>) {
    if let Ok(mut tool_text) = tool_text.get_single_mut() {
        tool_text.0 = format!("Tool: {}", *tool);
    }
}

/// Bevy [`Update`] system to turn on/off debugging when the player pressed D
#[cfg(feature = "debug")]
pub fn toggle_debug(
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugMenuState>,
    mut event_writer: EventWriter<ToggleDebugMenu>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        debug.toggle();
        event_writer.send(ToggleDebugMenu);
    }
}

/// Bevy [`Update`] system to handle toggling the debug menu
#[cfg(feature = "debug")]
pub fn toggle_debug_menu(
    mut query: Query<&mut Visibility, With<DebugMenu>>,
    debug_menu_state: Res<DebugMenuState>,
    mut event_reader: EventReader<ToggleDebugMenu>,
) {
    if let Ok(mut visibility) = query.get_single_mut() {
        for _ in event_reader.read() {
            match *debug_menu_state {
                DebugMenuState::On => *visibility = Visibility::Inherited,
                DebugMenuState::Off => *visibility = Visibility::Hidden,
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
    debug_menu_state: Res<DebugMenuState>,
) {
    if matches!(*debug_menu_state, DebugMenuState::Off) {
        return;
    }

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
pub fn setup_particle_count_text(
    mut commands: Commands,
    theme: Res<CatppuccinTheme>,
    debug_menu_state: Res<DebugMenuState>,
) {
    let menu_visibility = match *debug_menu_state {
        DebugMenuState::On => Visibility::Inherited,
        DebugMenuState::Off => Visibility::Hidden,
    };

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                column_gap: Val::Px(10.0),
                ..default()
            },
            DebugMenu,
            PickingBehavior::IGNORE,
            menu_visibility,
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
