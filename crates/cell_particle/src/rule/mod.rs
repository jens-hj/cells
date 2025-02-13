use percentage::Percentage;

use crate::grid::{Dimensions, Grid};

/// A type similar to [`Option`], but with a few extra tricks
#[derive(Debug, Clone)]
pub enum Occupancy<T> {
    /// The cell is occupied by `T`, should be thought of as [`Option::Some`]
    OccupiedBy(T),
    /// The cell is occupied by an unspecified, same as pattern matching `Occupancy::OccupiedBy(_)`,
    /// but it's all handled by this type
    OccupiedByAny,
    /// We don't care whether the cell is occupied or not, same as pattern matching `_`
    Unknown,
    /// The cell is not occupied, should be thought of as [`Option::None`]
    Vacant,
}

impl<T: PartialEq> PartialEq for Occupancy<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Occupancy::OccupiedBy(a), Occupancy::OccupiedBy(b)) => a == b,
            (Occupancy::OccupiedByAny, Occupancy::OccupiedByAny) => true,
            (Occupancy::OccupiedBy(_), Occupancy::OccupiedByAny) => true,
            (Occupancy::OccupiedByAny, Occupancy::OccupiedBy(_)) => true,
            (Occupancy::Unknown, _) => true,
            (_, Occupancy::Unknown) => true,
            (Occupancy::Vacant, Occupancy::Vacant) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Input<T: Clone + PartialEq + std::fmt::Debug> {
    pub grid: Grid<T>,
}

#[derive(Debug, Clone)]
pub struct Output<T: Clone + PartialEq + std::fmt::Debug> {
    pub grid: Grid<T>,
    pub probability: Percentage,
}

/// Error type for rule validation
#[derive(Debug)]
pub enum RuleError {
    /// Mismatch between the dimensions of the input and output grids
    DimensionMismatch {
        output_dims: Dimensions,
        input_dims: Dimensions,
    },
    /// Mismatch between the probabilities of the outputs
    OutputNotInProbabilisticUnity { total_probability: Percentage },
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleError::DimensionMismatch {
                output_dims,
                input_dims,
            } => {
                write!(
                    f,
                    "Output grid dimensions {} do not match input dimensions {}",
                    output_dims, input_dims
                )
            }
            RuleError::OutputNotInProbabilisticUnity { total_probability } => {
                write!(
                    f,
                    "Output probabilities do not form a unity: {}",
                    total_probability
                )
            }
        }
    }
}

impl std::error::Error for RuleError {}

/// A rule that defines the transformation of a specific grid state to a new grid state
/// multiple possible outputs can be defined, each with a different probability, all
/// probabilities must form a unity.
#[derive(Debug, Clone)]
pub struct Rule<T: Clone + PartialEq + std::fmt::Debug> {
    pub input: Input<T>,
    pub output: Vec<Output<T>>,
}

impl<T: Clone + PartialEq + std::fmt::Debug> Rule<T> {
    /// Validates the rule, following the following rules:
    /// - All output grids must match the dimensions of the input grid
    /// - All probabilities must form a unity
    pub fn validate(&self) -> Result<(), RuleError> {
        // Dimension validation
        let input_dims = self.input.grid.dimensions();
        for output in &self.output {
            let output_dims = output.grid.dimensions();
            if output_dims != input_dims {
                return Err(RuleError::DimensionMismatch {
                    output_dims,
                    input_dims,
                });
            }
        }

        // Probability validation
        let total_probability = self
            .output
            .iter()
            .map(|o| o.probability)
            .sum::<Percentage>();
        if !total_probability.is_one() {
            return Err(RuleError::OutputNotInProbabilisticUnity { total_probability });
        }

        Ok(())
    }

    /// Creates a new rule and validates the grid dimensions
    pub fn new(input: Input<T>, output: Vec<Output<T>>) -> Result<Self, RuleError> {
        let rule = Rule { input, output };
        rule.validate()?;
        Ok(rule)
    }

    /// Get the dimensions of the rule
    pub fn dimensions(&self) -> Dimensions {
        self.input.grid.dimensions()
    }

    /// Check if the rule matches on the given grid.
    /// The rule matches if the input grid matches the rule's input grid.
    pub fn matches(&self, grid: &Grid<T>) -> bool {
        if self.input.grid.dimensions() != grid.dimensions() {
            return false;
        }

        // Check if the input grid matches the rule's input grid
        for (i, row) in self.input.grid.cells.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if cell != &grid.cells[i][j] {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::particle::{Particle, ParticleKind};

    use super::*;

    #[test]
    fn test_rule_validation() {
        // Create the input and outputs
        let input = Input {
            grid: Grid::new(vec![vec![Particle::new(ParticleKind::Sand)]]).unwrap(),
        };
        let output = vec![Output {
            grid: Grid::new(vec![vec![Particle::new(ParticleKind::Sand)]]).unwrap(),
            probability: Percentage::new(1.0),
        }];

        // Validate the rule
        let rule = Rule::new(input, output).unwrap();

        // Check the dimensions of the input
        assert_eq!(
            rule.input.grid.dimensions(),
            Dimensions {
                width: 1,
                height: 1
            }
        );
    }
}
