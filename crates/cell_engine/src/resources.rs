use bevy::prelude::*;
use cell_particle::particle::ParticleKind;

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

/// Bevy [`Resource`] to turn on/off debugging.
/// Future: Have multiple debug menus of varying complexity or categories
#[cfg(feature = "debug")]
#[derive(Resource, Debug, Clone)]
pub enum DebugMenuState {
    /// Turn on/off debugging
    On,
    /// Turn off debugging
    Off,
}

#[cfg(feature = "debug")]
impl Default for DebugMenuState {
    fn default() -> Self {
        Self::On
    }
}

#[cfg(feature = "debug")]
impl DebugMenuState {
    pub fn toggle(&mut self) {
        match self {
            Self::On => *self = Self::Off,
            Self::Off => *self = Self::On,
        }
    }
}

/// Bevy [`Resource`] to keep track of which tool is currently selected
#[derive(Resource, Debug, Clone)]
pub enum Tool {
    /// The tool to select the content of a cell
    Despawn,
    /// The tool to spawn a particle
    Spawn(ParticleKind),
}

impl Default for Tool {
    fn default() -> Self {
        Self::Spawn(ParticleKind::Sand)
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // match self {
        //     Tool::Despawn => write!(f, "Despawn"),
        //     Tool::Spawn(kind) => write!(f, "Spawn {}", kind),
        // }
    }
}
