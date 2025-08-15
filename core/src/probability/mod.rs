use crate::Result;

/// Kolmogorov axioms implementation
/// Reference: Kolmogorov, A.N. (1933). "Grundbegriffe der Wahrscheinlichkeitsrechnung"
pub trait ProbabilitySpace {
    type Outcome;
    type Event;

    /// Axiom 1: Non-negativity - P(A) >= 0 for all events A
    fn verify_non_negativity(&self, event: &Self::Event) -> Result<()>;

    /// Axiom 2: Normalization - P(Ω) = 1 where Ω is the sample space
    fn verify_normalization(&self) -> Result<()>;

    /// Axiom 3: Countable additivity - For disjoint events, P(∪Ai) = ΣP(Ai)
    fn verify_countable_additivity(&self, events: &[Self::Event]) -> Result<()>;

    /// Calculate probability of an event
    fn probability(&self, event: &Self::Event) -> Result<f64>;

    /// Check if two events are disjoint
    fn are_disjoint(&self, event1: &Self::Event, event2: &Self::Event) -> bool;
}

/// Discrete probability distribution
#[derive(Debug, Clone)]
pub struct DiscreteDistribution {
    outcomes: Vec<String>,
    probabilities: Vec<f64>,
}

impl DiscreteDistribution {
    /// Create a new discrete distribution
    /// Probabilities must sum to 1.0 within tolerance
    pub fn new(outcomes: Vec<String>, probabilities: Vec<f64>) -> Result<Self> {
        if outcomes.len() != probabilities.len() {
            return Err(crate::PbjbiError::InvalidInput(
                "Outcomes and probabilities must have same length".to_string(),
            ));
        }

        // Verify non-negativity
        for p in &probabilities {
            if *p < 0.0 {
                return Err(crate::PbjbiError::Statistical(format!(
                    "Probability {} violates non-negativity axiom",
                    p
                )));
            }
        }

        // Verify normalization
        let sum: f64 = probabilities.iter().sum();
        if (sum - 1.0).abs() > 1e-10 {
            return Err(crate::PbjbiError::Statistical(format!(
                "Probabilities sum to {} instead of 1.0",
                sum
            )));
        }

        Ok(Self {
            outcomes,
            probabilities,
        })
    }

    /// Get probability of a specific outcome
    pub fn probability_of(&self, outcome: &str) -> Option<f64> {
        self.outcomes
            .iter()
            .position(|o| o == outcome)
            .map(|i| self.probabilities[i])
    }

    /// Calculate entropy H(X) = -Σ p(x) log p(x)
    pub fn entropy(&self) -> f64 {
        self.probabilities
            .iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| -p * p.log2())
            .sum()
    }

    /// Calculate expected value for numeric outcomes
    pub fn expected_value(&self) -> Result<f64> {
        let mut sum = 0.0;
        for (outcome, prob) in self.outcomes.iter().zip(&self.probabilities) {
            let value = outcome.parse::<f64>().map_err(|_| {
                crate::PbjbiError::InvalidInput(format!(
                    "Cannot parse outcome '{}' as number",
                    outcome
                ))
            })?;
            sum += value * prob;
        }
        Ok(sum)
    }
}

/// Probability measure for continuous distributions
pub struct ContinuousMeasure {
    /// Probability density function
    pdf: Box<dyn Fn(f64) -> f64>,
    /// Cumulative distribution function
    cdf: Box<dyn Fn(f64) -> f64>,
    /// Support of the distribution (min, max)
    #[allow(dead_code)]
    support: (f64, f64),
}

impl ContinuousMeasure {
    /// Create a uniform distribution on [a, b]
    pub fn uniform(a: f64, b: f64) -> Result<Self> {
        if a >= b {
            return Err(crate::PbjbiError::InvalidInput(format!(
                "Invalid uniform distribution bounds: [{}, {}]",
                a, b
            )));
        }

        let width = b - a;
        Ok(Self {
            pdf: Box::new(move |x| if x >= a && x <= b { 1.0 / width } else { 0.0 }),
            cdf: Box::new(move |x| {
                if x < a {
                    0.0
                } else if x > b {
                    1.0
                } else {
                    (x - a) / width
                }
            }),
            support: (a, b),
        })
    }

    /// Evaluate probability density at a point
    pub fn density(&self, x: f64) -> f64 {
        (self.pdf)(x)
    }

    /// Evaluate cumulative distribution at a point
    pub fn cumulative(&self, x: f64) -> f64 {
        (self.cdf)(x)
    }

    /// Calculate probability of interval [a, b]
    pub fn probability_interval(&self, a: f64, b: f64) -> f64 {
        self.cumulative(b) - self.cumulative(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_distribution_valid() {
        let dist = DiscreteDistribution::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![0.3, 0.5, 0.2],
        )
        .unwrap();

        assert_eq!(dist.probability_of("B"), Some(0.5));
        assert_eq!(dist.probability_of("D"), None);
    }

    #[test]
    fn test_discrete_distribution_invalid_sum() {
        let result = DiscreteDistribution::new(
            vec!["A".to_string(), "B".to_string()],
            vec![0.3, 0.5], // Sum is 0.8, not 1.0
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_entropy_calculation() {
        // Uniform distribution has maximum entropy
        let uniform = DiscreteDistribution::new(
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
            vec![0.25, 0.25, 0.25, 0.25],
        )
        .unwrap();

        assert!((uniform.entropy() - 2.0).abs() < 1e-10); // log2(4) = 2
    }

    #[test]
    fn test_uniform_distribution() {
        let uniform = ContinuousMeasure::uniform(0.0, 1.0).unwrap();

        assert_eq!(uniform.density(0.5), 1.0);
        assert_eq!(uniform.density(-0.5), 0.0);
        assert_eq!(uniform.cumulative(0.5), 0.5);
        assert!((uniform.probability_interval(0.2, 0.7) - 0.5).abs() < 1e-10);
    }
}
