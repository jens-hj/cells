use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum ParticleKind {
    Sand,
    Water,
    Stone,
}
