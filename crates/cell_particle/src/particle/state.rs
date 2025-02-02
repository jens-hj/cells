use super::ParticleKind;

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleState {
    /// Temperature in degrees Celsius
    pub temperature: f32,
    /// Pressure in kilo Pascals
    pub pressure: f32,
    /// Density in grams per cubic centimeter
    pub density: f32,
}

impl ParticleState {
    /// Creates a new particle state with the given temperature, pressure, and density
    pub fn new(temperature: f32, pressure: f32, density: f32) -> Self {
        Self { temperature, pressure, density }
    }

    /// Creates a default particle state from the given particle kind
    pub fn from_kind(kind: ParticleKind) -> Self {
        match kind {
            ParticleKind::Sand => ParticleState {
                density: 1.5,
                ..Default::default()
            },
            ParticleKind::Water => ParticleState {
                density: 1.0,
                ..Default::default()
            },
            ParticleKind::Stone => ParticleState {
                density: 2.65,
                ..Default::default()
            },
        }
    }
}

impl Default for ParticleState {
    /// Creates a default particle state with room temperature, atmospheric pressure, and standard density
    fn default() -> Self {
        Self { temperature: 20.0, pressure: 101.325, density: 1.0 }
    }
}
