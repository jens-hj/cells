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

#[derive(Debug, Clone)]
pub enum Occupancy<T> {
    Occupied(T),
    OccupiedAny,
    OccupiedOrEmpty,
    Empty,
}

impl<T> Occupancy<T> {
    pub fn map<U, F>(self, f: F) -> Occupancy<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Occupancy::Occupied(value) => Occupancy::Occupied(f(value)),
            Occupancy::OccupiedAny => Occupancy::OccupiedAny,
            Occupancy::OccupiedOrEmpty => Occupancy::OccupiedOrEmpty,
            Occupancy::Empty => Occupancy::Empty,
        }
    }
}

impl<T> PartialEq for Occupancy<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Direct comparison
            (Occupancy::Occupied(_), Occupancy::Occupied(_)) => true,
            (Occupancy::OccupiedAny, Occupancy::OccupiedAny) => true,
            (Occupancy::OccupiedOrEmpty, Occupancy::OccupiedOrEmpty) => true,
            (Occupancy::Empty, Occupancy::Empty) => true,
            // Occupied with OccupiedAny
            (Occupancy::Occupied(_), Occupancy::OccupiedAny) => true,
            (Occupancy::OccupiedAny, Occupancy::Occupied(_)) => true,
            // Occupied with OccupiedOrEmpty
            (Occupancy::Occupied(_), Occupancy::OccupiedOrEmpty) => true,
            (Occupancy::OccupiedOrEmpty, Occupancy::Occupied(_)) => true,
            // OccupiedOrEmpty with OccupiedAny
            (Occupancy::OccupiedOrEmpty, Occupancy::OccupiedAny) => true,
            (Occupancy::OccupiedAny, Occupancy::OccupiedOrEmpty) => true,
            // OccupiedOrEmpty with Empty
            (Occupancy::OccupiedOrEmpty, Occupancy::Empty) => true,
            (Occupancy::Empty, Occupancy::OccupiedOrEmpty) => true,
            _ => false,
        }
    }
}

// convert Occupancy to Option
impl<T> From<Occupancy<T>> for Option<T> {
    fn from(occupancy: Occupancy<T>) -> Self {
        match occupancy {
            Occupancy::Occupied(value) => Some(value),
            Occupancy::OccupiedAny => None,
            Occupancy::OccupiedOrEmpty => None,
            Occupancy::Empty => None,
        }
    }
}

impl<T> AsRef<T> for Occupancy<T> {
    fn as_ref(&self) -> &T {
        match self {
            Occupancy::Occupied(value) => value,
            Occupancy::OccupiedAny => panic!("Cannot get reference from OccupiedAny"),
            Occupancy::Empty => panic!("Cannot get reference from Empty"),
            Occupancy::OccupiedOrEmpty => panic!("Cannot get reference from OccupiedOrEmpty"),
        }
    }
}

/// Bevy [`Component`] for a cellular automaton rule
#[derive(Component, Debug, Clone)]
pub struct CellRule {
    pub rule: Rule<Occupancy<ParticleKind>>,
}

/// Wrapper cell for [`Particle`], which optionally contains a [`Particle`], and can tell you its color
#[derive(Debug, Clone)]
pub struct ParticleCell {
    pub content: Occupancy<Particle>,
}

impl ParticleCell {
    pub fn color(&self, flavor: &Flavor) -> Color {
        match &self.content {
            Occupancy::Occupied(particle) => match particle.kind {
                ParticleKind::Sand => flavor.yellow,
                ParticleKind::Water => flavor.blue,
                ParticleKind::Stone => flavor.surface2,
            },
            Occupancy::OccupiedAny => flavor.pink,
            Occupancy::OccupiedOrEmpty => flavor.pink,
            Occupancy::Empty => Color::NONE,
        }
    }
}

impl Default for ParticleCell {
    fn default() -> Self {
        ParticleCell {
            content: Occupancy::Empty,
        }
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
            cell.content = Occupancy::Occupied(particle::Particle::new(particle_kind.clone()));
        }
        self
    }

    pub fn with_random_particles(mut self) -> Self {
        for cell in self.grid.iter_mut() {
            let particle_kinds = ParticleKind::iter().collect::<Vec<_>>();
            let random_index = rand::rng().random_range(0..particle_kinds.len());
            let particle_kind = particle_kinds[random_index].clone();
            cell.content = Occupancy::Occupied(particle::Particle::new(particle_kind));
        }
        self
    }

    pub fn update(&mut self, rules: &Vec<Rule<Occupancy<ParticleKind>>>) {
        // update the grid for each rule
        for rule in rules {
            // loop over windows of the grid in the rule's width/height
            let rule_dimensions = rule.dimensions();

            // Create a grid of particle kinds from the cells
            let particle_kind_grid: Grid<Occupancy<ParticleKind>> = self
                .grid
                .iter()
                .map(|cell| cell.content.clone().map(|p| p.kind.clone()))
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
