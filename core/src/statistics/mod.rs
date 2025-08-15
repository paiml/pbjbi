use crate::{PbjbiError, Result};
use ndarray::Array1;
use statrs::distribution::{ContinuousCDF, Normal};

/// Descriptive statistics for numerical data
pub struct DescriptiveStats {
    data: Array1<f64>,
    sorted: Option<Array1<f64>>,
}

impl DescriptiveStats {
    pub fn new(data: Vec<f64>) -> Result<Self> {
        if data.is_empty() {
            return Err(PbjbiError::InvalidInput("Data cannot be empty".to_string()));
        }

        Ok(Self {
            data: Array1::from_vec(data),
            sorted: None,
        })
    }

    /// Mean (first moment)
    pub fn mean(&self) -> f64 {
        self.data.mean().unwrap_or(0.0)
    }

    /// Variance (second central moment)
    pub fn variance(&self) -> f64 {
        let mean = self.mean();
        let n = self.data.len() as f64;

        if n <= 1.0 {
            return 0.0;
        }

        self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)
    }

    /// Standard deviation
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Skewness (third standardized moment)
    /// Reference: Joanes, D.N. and Gill, C.A. (1998)
    pub fn skewness(&self) -> f64 {
        let n = self.data.len() as f64;
        if n < 3.0 {
            return 0.0;
        }

        let mean = self.mean();
        let std = self.std_dev();

        if std == 0.0 {
            return 0.0;
        }

        let m3 = self
            .data
            .iter()
            .map(|&x| ((x - mean) / std).powi(3))
            .sum::<f64>()
            / n;

        // Fisher-Pearson standardized moment coefficient
        m3 * (n * (n - 1.0)).sqrt() / (n - 2.0)
    }

    /// Kurtosis (fourth standardized moment)
    /// Returns excess kurtosis (normal distribution = 0)
    pub fn kurtosis(&self) -> f64 {
        let n = self.data.len() as f64;
        if n < 4.0 {
            return 0.0;
        }

        let mean = self.mean();
        let std = self.std_dev();

        if std == 0.0 {
            return 0.0;
        }

        let m4 = self
            .data
            .iter()
            .map(|&x| ((x - mean) / std).powi(4))
            .sum::<f64>()
            / n;

        // Excess kurtosis
        m4 - 3.0
    }

    /// Median (robust measure of central tendency)
    pub fn median(&mut self) -> f64 {
        self.ensure_sorted();
        let sorted = self.sorted.as_ref().unwrap();
        let n = sorted.len();

        if n % 2 == 0 {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        } else {
            sorted[n / 2]
        }
    }

    /// Median Absolute Deviation (robust measure of spread)
    /// Reference: Hampel, F.R. (1974)
    pub fn mad(&mut self) -> f64 {
        let median = self.median();
        let deviations: Vec<f64> = self.data.iter().map(|&x| (x - median).abs()).collect();

        let mut dev_stats = DescriptiveStats::new(deviations).unwrap();
        dev_stats.median() * 1.4826 // Scale factor for consistency with std dev
    }

    /// Quantile calculation using linear interpolation
    pub fn quantile(&mut self, q: f64) -> Result<f64> {
        if !(0.0..=1.0).contains(&q) {
            return Err(PbjbiError::InvalidInput(format!(
                "Quantile must be in [0, 1], got {}",
                q
            )));
        }

        self.ensure_sorted();
        let sorted = self.sorted.as_ref().unwrap();
        let n = sorted.len();

        if n == 1 {
            return Ok(sorted[0]);
        }

        let index = q * (n - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;
        let weight = index - lower as f64;

        Ok(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
    }

    fn ensure_sorted(&mut self) {
        if self.sorted.is_none() {
            let mut sorted = self.data.clone();
            sorted
                .as_slice_mut()
                .unwrap()
                .sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.sorted = Some(sorted);
        }
    }
}

/// Correlation methods
pub struct Correlation;

impl Correlation {
    /// Pearson correlation coefficient
    /// Reference: Pearson, K. (1895)
    pub fn pearson(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(PbjbiError::InvalidInput(
                "Arrays must have same length".to_string(),
            ));
        }

        if x.len() < 2 {
            return Err(PbjbiError::InvalidInput(
                "Need at least 2 points for correlation".to_string(),
            ));
        }

        let n = x.len() as f64;
        let x_mean = x.iter().sum::<f64>() / n;
        let y_mean = y.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut x_var = 0.0;
        let mut y_var = 0.0;

        for i in 0..x.len() {
            let x_dev = x[i] - x_mean;
            let y_dev = y[i] - y_mean;
            cov += x_dev * y_dev;
            x_var += x_dev * x_dev;
            y_var += y_dev * y_dev;
        }

        if x_var == 0.0 || y_var == 0.0 {
            return Ok(0.0); // No variation means no correlation
        }

        Ok(cov / (x_var * y_var).sqrt())
    }

    /// Spearman rank correlation
    /// Reference: Spearman, C. (1904)
    pub fn spearman(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(PbjbiError::InvalidInput(
                "Arrays must have same length".to_string(),
            ));
        }

        let x_ranks = Self::rank(x);
        let y_ranks = Self::rank(y);

        Self::pearson(&x_ranks, &y_ranks)
    }

    /// Calculate ranks with average rank for ties
    fn rank(data: &[f64]) -> Vec<f64> {
        let n = data.len();
        let mut indexed: Vec<(usize, f64)> =
            data.iter().enumerate().map(|(i, &v)| (i, v)).collect();

        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut ranks = vec![0.0; n];
        let mut i = 0;

        while i < n {
            let mut j = i;
            while j < n && indexed[j].1 == indexed[i].1 {
                j += 1;
            }

            let avg_rank = (i + j) as f64 / 2.0 + 0.5;
            for k in i..j {
                ranks[indexed[k].0] = avg_rank;
            }

            i = j;
        }

        ranks
    }
}

/// Statistical hypothesis testing
pub struct HypothesisTest;

impl HypothesisTest {
    /// One-sample t-test
    /// Reference: Student (1908)
    pub fn one_sample_t_test(data: &[f64], mu0: f64) -> Result<TTestResult> {
        let stats = DescriptiveStats::new(data.to_vec())?;
        let n = data.len() as f64;

        if n < 2.0 {
            return Err(PbjbiError::Statistical(
                "Need at least 2 samples for t-test".to_string(),
            ));
        }

        let mean = stats.mean();
        let std = stats.std_dev();

        if std == 0.0 {
            return Err(PbjbiError::Statistical(
                "Standard deviation is zero".to_string(),
            ));
        }

        let t_statistic = (mean - mu0) / (std / n.sqrt());
        let df = n - 1.0;

        // Approximate p-value using normal distribution for large samples
        // For exact p-value, would need t-distribution
        let normal = Normal::new(0.0, 1.0).unwrap();
        let p_value = 2.0 * (1.0 - normal.cdf(t_statistic.abs()));

        Ok(TTestResult {
            t_statistic,
            degrees_of_freedom: df,
            p_value,
            mean_difference: mean - mu0,
            confidence_interval: (mean - 1.96 * std / n.sqrt(), mean + 1.96 * std / n.sqrt()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TTestResult {
    pub t_statistic: f64,
    pub degrees_of_freedom: f64,
    pub p_value: f64,
    pub mean_difference: f64,
    pub confidence_interval: (f64, f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut stats = DescriptiveStats::new(data).unwrap();

        assert_eq!(stats.mean(), 3.0);
        assert!((stats.variance() - 2.5).abs() < 1e-10);
        assert_eq!(stats.median(), 3.0);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = Correlation::pearson(&x, &y).unwrap();
        assert!((corr - 1.0).abs() < 1e-10); // Perfect positive correlation
    }

    #[test]
    fn test_spearman_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // y = x^2

        let spearman = Correlation::spearman(&x, &y).unwrap();
        assert!((spearman - 1.0).abs() < 1e-10); // Perfect monotonic relationship
    }
}
