mod kind;
mod state;

pub use kind::ParticleKind;
pub use state::ParticleState;

#[derive(Debug, Clone)]
pub struct Particle {
    pub kind: ParticleKind,
    pub state: ParticleState,
}       

impl Particle {
    pub fn new(kind: ParticleKind) -> Self {
        let state = ParticleState::from_kind(kind.clone());
        Self { kind, state }
    }
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for Particle {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_particle() {
        let particle = Particle::new(ParticleKind::Sand);
        assert_eq!(particle.kind, ParticleKind::Sand);
        assert_eq!(particle.state, ParticleState::from_kind(ParticleKind::Sand));
    }

    #[test]
    fn test_particle_equality() {
        let p1 = Particle::new(ParticleKind::Sand);
        let p2 = Particle::new(ParticleKind::Sand);
        let p3 = Particle::new(ParticleKind::Water);

        // Particles are equal if they have the same kind, regardless of state
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);

        // Even with different states, same kinds are equal
        let mut p4 = Particle::new(ParticleKind::Sand);
        p4.state.temperature = 100.0;
        assert_eq!(p1, p4);
    }

    #[test]
    fn test_particle_state_initialization() {
        let sand = Particle::new(ParticleKind::Sand);
        let water = Particle::new(ParticleKind::Water);
        let stone = Particle::new(ParticleKind::Stone);

        // Check that each particle type gets the correct default density
        assert_eq!(sand.state.density, 1.5);
        assert_eq!(water.state.density, 1.0);
        assert_eq!(stone.state.density, 2.65);

        // Check that default temperature and pressure are set
        assert_eq!(sand.state.temperature, 20.0);
        assert_eq!(sand.state.pressure, 101.325);
    }
}
