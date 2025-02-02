use std::{iter::Sum, ops::{Add, Mul, Sub}};

/// Represents a percentage value between 0% and 100%
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f32);

pub const EPSILON: f32 = 1e-5;

impl Percentage {
    /// Creates a new Percentage from a value, clamping it to the range [0.0, 1.0]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Creates a new Percentage from a percent value, clamping it to the range [0, 100]
    pub fn from_percent(percent: f32) -> Self {
        Self::new(percent / 100.0)
    }

    /// Returns the raw value between 0.0 and 1.0
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Returns the percentage value between 0 and 100
    pub fn as_percent(&self) -> f32 {
        self.0 * 100.0
    }

    /// Tells whether the percentage represents a unity. I.e. close enough to 100%
    pub fn is_one(&self) -> bool {
        (self.0 - 1.0).abs() < EPSILON
    }

    /// Tells whether the percentage represents a zero. I.e. close enough to 0%
    pub fn is_zero(&self) -> bool {
        (self.0 - 0.0).abs() < EPSILON
    }
}

impl Mul<f32> for Percentage {
    type Output = Percentage;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.0 * rhs)
    }
}

impl Mul for Percentage {
    type Output = Percentage;

    fn mul(self, rhs: Self) -> Self {
        Self::new(self.0 * rhs.0)
    }
}

impl Add for Percentage {
    type Output = Percentage;

    fn add(self, rhs: Self) -> Self {
        Self::new((self.0 + rhs.0).min(1.0))
    }
}

impl Sub for Percentage {
    type Output = Percentage;

    fn sub(self, rhs: Self) -> Self {
        Self::new((self.0 - rhs.0).max(0.0))
    }
}

impl Sum for Percentage {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self::new(iter.map(|p| p.0).sum())
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.as_percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_creation() {
        let p = Percentage::new(0.5);
        assert!((p.value() - 0.5).abs() < EPSILON);
        assert!((p.as_percent() - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_from_percent() {
        let p = Percentage::from_percent(75.0);
        assert!((p.value() - 0.75).abs() < EPSILON);
        assert!((p.as_percent() - 75.0).abs() < EPSILON);
    }

    #[test]
    fn test_multiplication() {
        let p = Percentage::new(0.3);
        assert!((p * 2.0).value() - 0.6 < EPSILON);
    }

    #[test]
    fn test_multiplication_overflow() {
        let p = Percentage::new(0.7);
        assert!((p * 200.0).is_one());
    }

    #[test]
    fn test_addition() {
        let p1 = Percentage::new(0.3);
        let p2 = Percentage::new(0.4);
        let sum = p1 + p2;
        assert!((sum.value() - 0.7).abs() < EPSILON);
    }

    #[test]
    fn test_addition_overflow() {
        let p1 = Percentage::new(0.7);
        let p2 = Percentage::new(0.8);
        let sum = p1 + p2;
        assert!((sum.value() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_subtraction() {
        let p1 = Percentage::new(0.7);
        let p2 = Percentage::new(0.4);
        let diff = p1 - p2;
        assert!((diff.value() - 0.3).abs() < EPSILON);
    }

    #[test]
    fn test_subtraction_overflow() {
        let p1 = Percentage::new(0.7);
        let p2 = Percentage::new(0.8);
        let diff = p1 - p2;
        assert!((diff.value() - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_clamping() {
        // Sanity checks with [`Percentage::new`]
        assert_eq!(Percentage::new(1.5).value(), 1.0);
        assert_eq!(Percentage::new(-0.5).value(), 0.0);
        // Sanity checks with [`Percentage::from_percent`]
        assert_eq!(Percentage::from_percent(150.0).value(), 1.0);
        assert_eq!(Percentage::from_percent(-50.0).value(), 0.0);
    }

    #[test]
    fn test_is_one() {
        // Base case
        assert!(Percentage::new(1.0).is_one());
        // Test the epsilon
        assert!(Percentage::new(1.0 - EPSILON / 2.0).is_one());
        assert!(Percentage::new(1.0 + EPSILON / 2.0).is_one());
        // Levels of precision that should work
        assert!(!Percentage::from_percent(99.99).is_one());
        assert!(!Percentage::from_percent(0.01).is_one());
        // Sanity check
        assert!(!Percentage::new(0.5).is_one());
    }

    #[test]
    fn test_is_zero() {
        // Base case
        assert!(Percentage::new(0.0).is_zero());
        // Test the epsilon
        assert!(Percentage::new(0.0 + EPSILON / 2.0).is_zero());
        assert!(Percentage::new(0.0 - EPSILON / 2.0).is_zero());
        // Levels of precision that should work
        assert!(!Percentage::from_percent(0.01).is_zero());
        assert!(!Percentage::from_percent(99.99).is_zero());
        // Sanity check
        assert!(!Percentage::new(0.5).is_zero());
    }

    #[test]
    fn test_percentage_multiplication() {
        let p1 = Percentage::new(0.5);  // 50%
        let p2 = Percentage::new(0.6);  // 60%
        let result = p1 * p2;
        assert!((result.value() - 0.3).abs() < EPSILON);  // 50% of 60% = 30%
        
        // Test commutative property
        let result2 = p2 * p1;
        assert!((result2.value() - 0.3).abs() < EPSILON);

        // Test with 50% of 50% of 60% which should be 15%
        let result3 = p1 * p1 * p2;
        assert!((result3.value() - 0.15).abs() < EPSILON);
    }
}
