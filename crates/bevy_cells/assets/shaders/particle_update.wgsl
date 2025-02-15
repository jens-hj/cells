@group(0) @binding(0)
var<storage, read> input_grid: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_grid: array<u32>;

struct ParticleKind {
    EMPTY: u32,
    SAND: u32,
    WATER: u32,
    STONE: u32
}

const PARTICLE: ParticleKind = ParticleKind(0u, 1u, 2u, 3u);

struct Dimensions {
    width: u32,
    height: u32,
}

@group(0) @binding(2)
var<uniform> dimensions: Dimensions;

fn get_index(x: u32, y: u32) -> u32 {
    return (dimensions.height - 1u - y) * dimensions.width + x;
}

fn is_empty(x: u32, y: u32) -> bool {
    if (x >= dimensions.width || y >= dimensions.height) {
        return false;
    }
    return input_grid[get_index(x, y)] == PARTICLE.EMPTY;
}

fn try_move_particle(x: u32, y: u32, kind: u32) -> bool {
    // Don't move if at bottom
    if (y == 0u) {
        return false;
    }

    // Try moving down
    if (is_empty(x, y - 1u)) {
        output_grid[get_index(x, y - 1u)] = kind;
        output_grid[get_index(x, y)] = PARTICLE.EMPTY;
        return true;
    }

    // For sand and water, try moving diagonally
    if (kind == PARTICLE.SAND || kind == PARTICLE.WATER) {
        let rand = hash(vec2<f32>(f32(x), f32(y)));
        
        // Try left or right first based on random value
        let dx = select(-1, 1, rand > 0.5);
        
        if (x > 0u && is_empty(x - 1u, y - 1u)) {
            output_grid[get_index(x - 1u, y - 1u)] = kind;
            output_grid[get_index(x, y)] = PARTICLE.EMPTY;
            return true;
        }
        
        if (x < dimensions.width - 1u && is_empty(x + 1u, y - 1u)) {
            output_grid[get_index(x + 1u, y - 1u)] = kind;
            output_grid[get_index(x, y)] = PARTICLE.EMPTY;
            return true;
        }
    }

    // Additional water spread behavior
    if (kind == PARTICLE.WATER) {
        if (x > 0u && is_empty(x - 1u, y)) {
            output_grid[get_index(x - 1u, y)] = kind;
            output_grid[get_index(x, y)] = PARTICLE.EMPTY;
            return true;
        }
        
        if (x < dimensions.width - 1u && is_empty(x + 1u, y)) {
            output_grid[get_index(x + 1u, y)] = kind;
            output_grid[get_index(x, y)] = PARTICLE.EMPTY;
            return true;
        }
    }

    return false;
}

// Simple hash function for random values
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= dimensions.width || y >= dimensions.height) {
        return;
    }

    let index = get_index(x, y);
    let particle = input_grid[index];
    
    // Copy current state to output if no movement occurs
    output_grid[index] = particle;
    
    // Skip empty cells and stone
    if (particle == PARTICLE.EMPTY || particle == PARTICLE.STONE) {
        return;
    }

    _ = try_move_particle(x, y, particle);
} 