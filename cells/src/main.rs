use nannou::prelude::*;

// Cell states
#[derive(Debug, Clone, Copy, PartialEq)]
enum Particle {
    Empty,
    Sand,
}

impl Particle {
    fn color(&self) -> Srgb<u8> {
        match self {
            Particle::Empty => BLACK,
            Particle::Sand => Rgb8::new(249, 226, 175),
        }
    }
}

struct Model {
    grid: Vec<Particle>,
    width: usize,
    height: usize,
    cell_size: f32,
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
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Reset grid with R key
    if app.keys.down.contains(&Key::R) {
        model.grid.fill(Particle::Empty);
    }

    // Add sand with mouse ONLY when left button is held
    if app.mouse.buttons.left().is_down() {
        let win_rect = app.window_rect();
        let mouse_pos = app.mouse.position();

        // Convert mouse position to grid coordinates
        let grid_x = ((mouse_pos.x - win_rect.left()) / model.cell_size) as usize;
        let grid_y = ((mouse_pos.y - win_rect.bottom()) / model.cell_size) as usize;

        if grid_x < model.width && grid_y < model.height {
            let index = grid_y * model.width + grid_x;
            model.grid[index] = Particle::Sand;
        }
    }

    // Create new grid for double buffering
    let mut new_grid = model.grid.clone();

    // Process grid from TOP TO BOTTOM (crucial for proper falling)
    for y in (0..model.height).rev() {
        // REVERSED iteration
        for x in 0..model.width {
            let index = y * model.width + x;

            // Only process sand in the ORIGINAL grid
            if model.grid[index] != Particle::Sand {
                continue;
            }

            let mut new_x = x;
            let mut new_y = y;

            // Try to move down (decrease y since y=0 is bottom)
            if y > 0 {
                let below_index = (y - 1) * model.width + x;
                if model.grid[below_index] == Particle::Empty {
                    new_y = y - 1;
                }
            }

            // If couldn't move down, try diagonal directions
            if new_y == y && y > 0 {
                let directions = [-1, 1];
                for dx in directions {
                    let test_x = x as i32 + dx;
                    if test_x >= 0 && test_x < model.width as i32 {
                        let diag_index = (y - 1) * model.width + test_x as usize;
                        if model.grid[diag_index] == Particle::Empty {
                            new_x = test_x as usize;
                            new_y = y - 1;
                            break;
                        }
                    }
                }
            }

            // Update positions only if changed
            if new_y != y || new_x != x {
                let new_index = new_y * model.width + new_x;
                if new_grid[new_index] == Particle::Empty {
                    new_grid[index] = Particle::Empty;
                    new_grid[new_index] = Particle::Sand;
                }
            }
        }
    }

    model.grid = new_grid;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(Rgb8::new(30, 30, 46));

    let win_rect = app.window_rect();
    let cell_size = model.cell_size;

    for y in 0..model.height {
        for x in 0..model.width {
            let index = y * model.width + x;
            if model.grid[index] == Particle::Sand {
                // Correct position calculation without Y inversion
                let x_pos = win_rect.left() + x as f32 * cell_size + cell_size / 2.0;
                let y_pos = win_rect.bottom() + y as f32 * cell_size + cell_size / 2.0;

                // Add slight randomisation to the color
                let random_offset = 1.0 + app.time.sin() * 0.05;
                let (r, g, b) = model.grid[index].color().into_components();
                let color = Srgb::new(
                    (r as f32 * random_offset) as u8,
                    (g as f32 * random_offset) as u8,
                    (b as f32 * random_offset) as u8,
                );

                // Draw the cell
                draw.rect()
                    .x_y(x_pos, y_pos)
                    .w_h(cell_size, cell_size)
                    .color(color);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
