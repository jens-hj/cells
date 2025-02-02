use nannou::prelude::*;
use rand::Rng;

// Define a trait for particle behavior
trait ParticleRule {
    fn update(
        &self,
        x: usize,
        y: usize,
        grid: &[Particle],
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)>;
    fn color(&self) -> Srgb<u8>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Particle {
    Empty,
    Sand,
    Water(Option<i32>), // None means no direction, Some(-1) for left, Some(1) for right
    Stone,
}

// Implement specific rules for each particle type
struct SandRule;
impl ParticleRule for SandRule {
    fn update(
        &self,
        x: usize,
        y: usize,
        grid: &[Particle],
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        // Ignore height
        let _ = height;

        if y == 0 {
            return None;
        }

        // Try moving down
        let below_idx = (y - 1) * width + x;
        if grid[below_idx] == Particle::Empty {
            return Some((x, y - 1));
        }

        // Try moving diagonally
        let mut rng = rand::rng();
        let directions = if rng.random_bool(0.5) {
            [-1, 1]
        } else {
            [1, -1]
        };

        for dx in directions {
            let new_x = x as i32 + dx;
            if new_x >= 0 && new_x < width as i32 {
                let diag_idx = (y - 1) * width + new_x as usize;
                if grid[diag_idx] == Particle::Empty {
                    return Some((new_x as usize, y - 1));
                }
            }
        }

        None
    }

    fn color(&self) -> Srgb<u8> {
        Rgb8::new(249, 226, 175)
    }
}

struct WaterRule;
impl ParticleRule for WaterRule {
    fn update(
        &self,
        x: usize,
        y: usize,
        grid: &[Particle],
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        // Ignore height
        let _ = height;

        if y == 0 {
            return None;
        }

        // Try moving down
        let below_idx = (y - 1) * width + x;
        if grid[below_idx] == Particle::Empty {
            return Some((x, y - 1));
        }

        // Get current direction or initialize new one
        let current_direction = if let Particle::Water(dir) = grid[y * width + x] {
            dir
        } else {
            None
        };

        let direction = match current_direction {
            Some(dir) => dir,
            None => {
                let mut rng = rand::rng();
                if rng.random_bool(0.5) {
                    -1
                } else {
                    1
                }
            }
        };

        // Try moving in current direction
        let new_x = x as i32 + direction;
        if new_x >= 0 && new_x < width as i32 {
            let side_idx = y * width + new_x as usize;
            if grid[side_idx] == Particle::Empty {
                return Some((new_x as usize, y));
            }
        }

        // Try moving in opposite direction
        let new_x = x as i32 - direction;
        if new_x >= 0 && new_x < width as i32 {
            let side_idx = y * width + new_x as usize;
            if grid[side_idx] == Particle::Empty {
                return Some((new_x as usize, y));
            }
        }

        None
    }

    fn color(&self) -> Srgb<u8> {
        Rgb8::new(100, 149, 237)
    }
}

struct StoneRule;
impl ParticleRule for StoneRule {
    fn update(
        &self,
        x: usize,
        y: usize,
        grid: &[Particle],
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        // Ignore all arguments
        let _ = x;
        let _ = y;
        let _ = grid;
        let _ = width;
        let _ = height;

        // Stone doesn't move
        None
    }

    fn color(&self) -> Srgb<u8> {
        Rgb8::new(169, 169, 169)
    }
}

impl Particle {
    fn get_rule(&self) -> Option<Box<dyn ParticleRule>> {
        match self {
            Particle::Empty => None,
            Particle::Sand => Some(Box::new(SandRule)),
            Particle::Water(_) => Some(Box::new(WaterRule)),
            Particle::Stone => Some(Box::new(StoneRule)),
        }
    }

    fn color(&self) -> Srgb<u8> {
        match self.get_rule() {
            Some(rule) => rule.color(),
            None => BLACK,
        }
    }
}

struct Model {
    grid: Vec<Particle>,
    width: usize,
    height: usize,
    cell_size: f32,
    current_particle: Particle,
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

fn model(app: &App) -> Model {
    let window = app.main_window();
    let win_rect = window.rect();

    let cell_size = 4.0;
    let width = (win_rect.w() / cell_size) as usize;
    let height = (win_rect.h() / cell_size) as usize;

    Model {
        grid: vec![Particle::Empty; width * height],
        width,
        height,
        cell_size,
        current_particle: Particle::Sand, // Default particle type
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Handle particle selection
    if app.keys.down.contains(&Key::Key1) {
        model.current_particle = Particle::Sand;
    } else if app.keys.down.contains(&Key::Key2) {
        model.current_particle = Particle::Water(None);
    } else if app.keys.down.contains(&Key::Key3) {
        model.current_particle = Particle::Stone;
    }

    // Reset grid with R key
    if app.keys.down.contains(&Key::R) {
        model.grid.fill(Particle::Empty);
    }

    // Add particles with mouse
    if app.mouse.buttons.left().is_down() {
        let win_rect = app.window_rect();
        let mouse_pos = app.mouse.position();

        let grid_x = ((mouse_pos.x - win_rect.left()) / model.cell_size) as usize;
        let grid_y = ((mouse_pos.y - win_rect.bottom()) / model.cell_size) as usize;

        if grid_x < model.width && grid_y < model.height {
            let index = grid_y * model.width + grid_x;
            model.grid[index] = model.current_particle;
        }
    }

    // Create new grid for double buffering
    let mut new_grid = model.grid.clone();
    let mut targeted_cells = vec![false; model.width * model.height];

    // Process grid from bottom to top
    for y in (0..model.height).rev() {
        for x in 0..model.width {
            let index = y * model.width + x;
            let particle = model.grid[index];

            if let Some(rule) = particle.get_rule() {
                if let Some((new_x, new_y)) =
                    rule.update(x, y, &model.grid, model.width, model.height)
                {
                    let new_index = new_y * model.width + new_x;

                    // Only move if the target cell hasn't been targeted by another particle
                    if !targeted_cells[new_index] {
                        targeted_cells[new_index] = true;
                        // For water particles, preserve the direction when moving
                        if let Particle::Water(_) = particle {
                            // Get the current direction from the original position
                            if let Particle::Water(dir) = model.grid[index] {
                                new_grid[new_index] = Particle::Water(dir);
                            } else {
                                // Initialize with a random direction if none exists
                                let mut rng = rand::rng();
                                let dir = if rng.random_bool(0.5) {
                                    Some(-1)
                                } else {
                                    Some(1)
                                };
                                new_grid[new_index] = Particle::Water(dir);
                            }
                        } else {
                            new_grid[new_index] = particle;
                        }
                        new_grid[index] = Particle::Empty;
                    }
                }
            }
        }
    }

    model.grid = new_grid;
}

// [Previous code remains the same until the update function...]

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(Rgb8::new(30, 30, 46));

    let win_rect = app.window_rect();
    let cell_size = model.cell_size;

    // Draw current particle type indicator
    let indicator_text = match model.current_particle {
        Particle::Sand => "Sand (1)",
        Particle::Water(_) => "Water (2)",
        Particle::Stone => "Stone (3)",
        Particle::Empty => "Empty",
    };
    draw.text(indicator_text)
        .x_y(win_rect.left() + 100.0, win_rect.top() - 20.0)
        .color(WHITE);

    for y in 0..model.height {
        for x in 0..model.width {
            let index = y * model.width + x;
            if model.grid[index] != Particle::Empty {
                // Calculate position
                let x_pos = win_rect.left() + x as f32 * cell_size + cell_size / 2.0;
                let y_pos = win_rect.bottom() + y as f32 * cell_size + cell_size / 2.0;

                // Get base color from particle
                let base_color = model.grid[index].color();
                let (r, g, b) = base_color.into_components();

                // Add slight randomization to the color based on time
                let random_offset = 1.0 + (app.time.sin() * 0.05);
                let color = Srgb::new(
                    (r as f32 * random_offset) as u8,
                    (g as f32 * random_offset) as u8,
                    (b as f32 * random_offset) as u8,
                );

                // Draw the particle
                draw.rect()
                    .x_y(x_pos, y_pos)
                    .w_h(cell_size, cell_size)
                    .color(color);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
