use std::collections::HashSet;

use bevy::prelude::*;
use bevy_catppuccin::*;
use cell_particle::{
    grid::Grid,
    particle::{self, Particle, ParticleKind},
    rule::Rule,
};
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    Rng,
};
use strum::IntoEnumIterator;

/// Bevy [`Component`] for a cellular automaton rule
#[derive(Component, Debug, Clone)]
pub struct CellRule {
    pub rule: Rule<Option<ParticleKind>>,
}

/// Wrapper cell for [`Particle`], which optionally contains a [`Particle`], and can tell you its color
#[derive(Debug, Clone)]
pub struct ParticleCell {
    pub content: Option<Particle>,
}

impl ParticleCell {
    pub fn color(&self, flavor: &Flavor) -> Color {
        match &self.content {
            Some(particle) => match particle.kind {
                ParticleKind::Sand => flavor.yellow,
                ParticleKind::Water => flavor.blue,
                ParticleKind::Stone => flavor.surface2,
            },
            None => Color::NONE,
        }
    }
}

impl Default for ParticleCell {
    fn default() -> Self {
        ParticleCell { content: None }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveCells {
    pub cells: HashSet<(usize, usize)>,
    pub to_check_next_frame: HashSet<(usize, usize)>,
}

impl ActiveCells {
    pub fn new() -> Self {
        Self {
            cells: HashSet::new(),
            to_check_next_frame: HashSet::new(),
        }
    }

    pub fn mark_active(&mut self, x: usize, y: usize) {
        self.cells.insert((x, y));
    }

    pub fn mark_for_next_frame(&mut self, x: usize, y: usize) {
        self.to_check_next_frame.insert((x, y));
    }

    pub fn update(&mut self) {
        std::mem::swap(&mut self.cells, &mut self.to_check_next_frame);
        self.to_check_next_frame.clear();
    }
}

impl Default for ActiveCells {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy [`Component`] for the world, which is a [`Grid`] of [`Cell`]s
#[derive(Component, Debug, Clone)]
#[require(Transform)]
pub struct CellWorld {
    /// Physical resolution of the world in pixels per cell. Each cell is a square.
    pub resolution: u32,
    /// The data of the world itself, grid of cells
    pub grid: Grid<ParticleCell>,
    /// The cells that are active in the current frame
    pub active_cells: ActiveCells,
}

impl CellWorld {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Grid::new(vec![vec![ParticleCell::default(); width]; height]).unwrap();
        CellWorld {
            resolution: 10,
            grid,
            active_cells: ActiveCells::new(),
        }
    }

    pub fn with_resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_fill(mut self, particle_kind: ParticleKind) -> Self {
        for cell in self.grid.iter_mut() {
            cell.content = Some(particle::Particle::new(particle_kind.clone()));
        }
        self
    }

    pub fn with_random_particles(mut self) -> Self {
        for cell in self.grid.iter_mut() {
            let particle_kinds = ParticleKind::iter().collect::<Vec<_>>();
            let random_index = rand::rng().random_range(0..particle_kinds.len());
            let particle_kind = particle_kinds[random_index].clone();
            cell.content = Some(particle::Particle::new(particle_kind));
        }
        self
    }

    pub fn update(&mut self, rules: &Vec<Rule<Option<ParticleKind>>>) {
        let mut new_grid = self.grid.clone();
        let cells_to_check: Vec<_> = self.active_cells.cells.iter().cloned().collect();

        let mut next_active_cells = std::mem::take(&mut self.active_cells);

        // First, mark all cells that contain particles and their neighbors as active
        for &(x, y) in &cells_to_check {
            if let Ok(cell) = self.grid.get(x, y) {
                if cell.content.is_some() {
                    // If cell has a particle, mark it and its neighbors
                    for dy in y.saturating_sub(1)..=(y + 1) {
                        for dx in x.saturating_sub(1)..=(x + 1) {
                            if dy < self.grid.dimensions().height
                                && dx < self.grid.dimensions().width
                            {
                                next_active_cells.mark_for_next_frame(dx, dy);
                            }
                        }
                    }
                }
            }
        }

        // Process rules as before
        for &(x, y) in &cells_to_check {
            let min_x = x.saturating_sub(1);
            let min_y = y.saturating_sub(1);
            let max_x = (x + 2).min(self.grid.dimensions().width);
            let max_y = (y + 2).min(self.grid.dimensions().height);

            for rule in rules {
                let rule_dims = rule.dimensions();
                let window_width = rule_dims.width;
                let window_height = rule_dims.height;

                if min_x + window_width > max_x || min_y + window_height > max_y {
                    continue;
                }

                if let Ok(window) = self
                    .grid
                    .get_subgrid(min_x, min_y, window_width, window_height)
                {
                    let particle_kind_window: Grid<Option<ParticleKind>> = Grid::new(
                        window
                            .cells
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|cell| cell.content.as_ref().map(|p| p.kind.clone()))
                                    .collect()
                            })
                            .collect(),
                    )
                    .unwrap();

                    if rule.matches(&particle_kind_window) {
                        // info!("Rule matched at ({}, {})", x, y);
                        let chosen_output = self.choose_rule_output(rule);
                        new_grid.set_subgrid(min_x, min_y, chosen_output).unwrap();

                        // Mark the affected area and surroundings
                        for dy in min_y.saturating_sub(1)..=(min_y + window_height + 1) {
                            for dx in min_x.saturating_sub(1)..=(min_x + window_width) {
                                if dy < self.grid.dimensions().height
                                    && dx < self.grid.dimensions().width
                                {
                                    next_active_cells.mark_for_next_frame(dx, dy);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.grid = new_grid;
        self.active_cells = next_active_cells;
        self.active_cells.update();
    }

    fn choose_rule_output(&self, rule: &Rule<Option<ParticleKind>>) -> Grid<ParticleCell> {
        let weighted_index =
            WeightedIndex::new(rule.output.iter().map(|o| o.probability.value())).unwrap();
        let chosen_output = rule.output[weighted_index.sample(&mut rand::rng())].clone();

        // Convert to ParticleCell grid
        Grid::new(
            chosen_output
                .grid
                .cells
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| ParticleCell {
                            content: cell.clone().map(|kind| particle::Particle::new(kind)),
                        })
                        .collect()
                })
                .collect(),
        )
        .unwrap()
    }
}

impl Default for CellWorld {
    fn default() -> Self {
        CellWorld::new(100, 100)
    }
}

/// Bevy marker [`Component`] for the main camera
#[derive(Component, Debug, Clone)]
pub struct MainCamera;

/// Bevy marker [`Component`] for visualisation of any [`Entity`].
/// If the [`Entity`] has a [`View`] component, it will is being visualised.
#[derive(Component, Debug, Clone)]
pub struct View;

#[derive(Component, Debug, Clone)]
pub struct WorldTexture;
