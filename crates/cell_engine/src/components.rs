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

/// Bevy [`Component`] for the world, which is a [`Grid`] of [`Cell`]s
#[derive(Component, Debug, Clone)]
#[require(Transform)]
pub struct CellWorld {
    /// Physical resolution of the world in pixels per cell. Each cell is a square.
    pub resolution: u32,
    /// The data of the world itself, grid of cells
    pub grid: Grid<ParticleCell>,
}

impl CellWorld {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Grid::new(vec![vec![ParticleCell::default(); width]; height]).unwrap();
        CellWorld {
            resolution: 10,
            grid,
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
        // update the grid for each rule
        for rule in rules {
            // loop over windows of the grid in the rule's width/height
            let rule_dimensions = rule.dimensions();

            // Create a grid of particle kinds from the cells
            let particle_kind_grid: Grid<Option<ParticleKind>> = self
                .grid
                .iter()
                .map(|cell| cell.content.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
                .chunks(self.grid.dimensions().width)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            // Iterate over windows and check rule matches
            for window in particle_kind_grid.windowed(rule_dimensions) {
                if rule.matches(&window.grid) {
                    // Weighted choice of output grid, weighted by each output's probability in rule.output[i].probability
                    let weighted_index =
                        WeightedIndex::new(rule.output.iter().map(|o| o.probability.value()))
                            .unwrap();

                    let chosen_output =
                        rule.output[weighted_index.sample(&mut rand::rng())].clone();

                    // make output grid into a grid of particle cells
                    let output_grid: Grid<ParticleCell> = chosen_output
                        .grid
                        .iter()
                        .map(|cell| ParticleCell {
                            content: cell.clone().map(|kind| particle::Particle::new(kind)),
                        })
                        .collect::<Vec<_>>()
                        .chunks(chosen_output.grid.dimensions().width)
                        .map(|chunk| chunk.to_vec())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap();

                    // Apply rule output here
                    self.grid
                        .set_subgrid(window.x, window.y, output_grid)
                        .expect(
                        "Output of rule should not be bigger than the grid it is being applied to",
                    );
                }
            }
        }
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
