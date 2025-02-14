use std::collections::HashSet;

use bevy::prelude::*;
use bevy_catppuccin::*;
use cell_particle::{
    grid::Grid,
    particle::{self, Particle, ParticleKind},
    rule::{Occupancy, Rule},
};
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::SliceRandom,
    Rng,
};
use strum::IntoEnumIterator;

/// Bevy [`Component`] for a cellular automaton rule
#[derive(Component, Debug, Clone)]
pub struct CellRule {
    /// The rule to apply
    pub rule: Rule<Occupancy<ParticleKind>>,
    /// The priority of the rule, if not set, the rule doesn't care about the order of application, and will be randomly shuffled
    pub priority: Option<usize>,
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
    pub affected_this_frame: HashSet<(usize, usize)>,
}

impl ActiveCells {
    pub fn new() -> Self {
        Self {
            cells: HashSet::new(),
            to_check_next_frame: HashSet::new(),
            affected_this_frame: HashSet::new(),
        }
    }

    pub fn mark_active(&mut self, x: usize, y: usize) {
        self.cells.insert((x, y));
    }

    pub fn mark_for_next_frame(&mut self, x: usize, y: usize) {
        self.to_check_next_frame.insert((x, y));
    }

    pub fn mark_affected(&mut self, x: usize, y: usize) {
        self.affected_this_frame.insert((x, y));
    }

    pub fn update(&mut self) {
        std::mem::swap(&mut self.cells, &mut self.to_check_next_frame);
        self.to_check_next_frame.clear();
        self.affected_this_frame.clear();
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

    pub fn update(&mut self, rules: &Vec<CellRule>) {
        let mut new_grid = self.grid.clone();
        let cells_to_check: Vec<_> = self.active_cells.cells.iter().cloned().collect();
        let mut next_active_cells = std::mem::take(&mut self.active_cells);

        // Separate rules into prioritized and unprioritized
        let (prioritized, unprioritized): (Vec<_>, Vec<_>) =
            rules.iter().partition(|rule| rule.priority.is_some());

        // Sort prioritized rules by priority
        let mut ordered_rules: Vec<_> = prioritized.clone();
        ordered_rules.sort_by_key(|rule| rule.priority.unwrap());

        // Group and shuffle rules with same priority
        let mut prioritised_rules = Vec::with_capacity(rules.len());
        let mut current_group = Vec::new();
        let mut current_priority = None;

        for rule in ordered_rules {
            match current_priority {
                None => {
                    current_priority = Some(rule.priority.unwrap());
                    current_group.push(rule);
                }
                Some(prev) if prev != rule.priority.unwrap() => {
                    if !current_group.is_empty() {
                        current_group.shuffle(&mut rand::rng());
                        prioritised_rules.extend(current_group.drain(..));
                    }
                    current_priority = Some(rule.priority.unwrap());
                    current_group.push(rule);
                }
                _ => current_group.push(rule),
            }
        }
        // Handle the last group
        if !current_group.is_empty() {
            current_group.shuffle(&mut rand::rng());
            prioritised_rules.extend(current_group);
        }

        // Randomly insert unprioritized rules
        for rule in unprioritized {
            let insert_pos = rand::rng().random_range(0..=prioritised_rules.len());
            prioritised_rules.insert(insert_pos, rule);
        }

        // Process rules and track which cells were affected
        'cell_loop: for &(x, y) in &cells_to_check {
            // Skip if this cell has already been affected by a rule this frame
            if next_active_cells.affected_this_frame.contains(&(x, y)) {
                continue;
            }

            for rule in prioritised_rules
                .iter()
                .map(|r| r.rule.clone())
                .collect::<Vec<_>>()
            {
                let rule_dims = rule.dimensions();

                // Center the rule window on the particle
                let rule_x = x.saturating_sub(rule_dims.width / 2);
                let rule_y = y.saturating_sub(rule_dims.height / 2);

                if rule_x + rule_dims.width > self.grid.dimensions().width
                    || rule_y + rule_dims.height > self.grid.dimensions().height
                {
                    continue;
                }

                if let Ok(window) =
                    self.grid
                        .get_subgrid(rule_x, rule_y, rule_dims.width, rule_dims.height)
                {
                    let particle_kind_window: Grid<Occupancy<ParticleKind>> = Grid::new(
                        window
                            .cells
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|cell| {
                                        match cell.content.as_ref().map(|p| p.kind.clone()) {
                                            Some(kind) => Occupancy::OccupiedBy(kind),
                                            None => Occupancy::Vacant,
                                        }
                                    })
                                    .collect()
                            })
                            .collect(),
                    )
                    .unwrap();

                    if rule.matches(&particle_kind_window) {
                        let chosen_output = self.choose_rule_output(&rule, &window);
                        new_grid.set_subgrid(rule_x, rule_y, chosen_output).unwrap();

                        // Mark all cells in the rule window as affected
                        for dy in 0..rule_dims.height {
                            for dx in 0..rule_dims.width {
                                next_active_cells.mark_affected(rule_x + dx, rule_y + dy);
                            }
                        }

                        // Mark cells for next frame's active set
                        for dy in
                            y.saturating_sub(1)..=(y + 1).min(self.grid.dimensions().height - 1)
                        {
                            for dx in
                                x.saturating_sub(1)..=(x + 1).min(self.grid.dimensions().width - 1)
                            {
                                next_active_cells.mark_for_next_frame(dx, dy);

                                if dy + 1 < self.grid.dimensions().height {
                                    next_active_cells.mark_for_next_frame(dx, dy + 1);
                                }
                            }
                        }

                        continue 'cell_loop; // Skip remaining rules for this cell
                    }
                }
            }
        }

        self.grid = new_grid;
        self.active_cells = next_active_cells;
        self.active_cells.update();
    }

    fn choose_rule_output(
        &self,
        rule: &Rule<Occupancy<ParticleKind>>,
        current_grid_window: &Grid<ParticleCell>,
    ) -> Grid<ParticleCell> {
        let weighted_index =
            WeightedIndex::new(rule.output.iter().map(|o| o.probability.value())).unwrap();
        let chosen_output = rule.output[weighted_index.sample(&mut rand::rng())].clone();

        // Convert to ParticleCell grid
        Grid::new(
            chosen_output
                .grid
                .cells
                .iter()
                .enumerate()
                .map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(x, cell)| ParticleCell {
                            content: match cell {
                                Occupancy::OccupiedBy(kind) => {
                                    Some(particle::Particle::new(kind.clone()))
                                }
                                Occupancy::Unknown | Occupancy::OccupiedByAny => {
                                    current_grid_window.get(x, y).unwrap().content.clone()
                                }
                                _ => None,
                            },
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

/// Bevy marker [`Component`] for the texture of the world
#[derive(Component, Debug, Clone)]
pub struct WorldTexture;

/// Bevy marker [`Component`] for the text of the current tool
#[derive(Component, Debug, Clone)]
pub struct ToolText;

/// Bevy marker [`Component`] for the text of the number of spawned particles
#[cfg(feature = "debug")]
#[derive(Component, Debug, Clone)]
pub struct SpawnedParticleCountText;

/// Bevy marker [`Component`] for the text of the number of existing particles
#[cfg(feature = "debug")]
#[derive(Component, Debug, Clone)]
pub struct ExistingParticleCountText;

/// Bevy marker [`Component`] for the debug menu
#[cfg(feature = "debug")]
#[derive(Component, Debug, Clone)]
pub struct DebugMenu;
