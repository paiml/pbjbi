use crate::{PbjbiError, Result};
use std::collections::HashMap;

/// Shewhart control charts with Western Electric rules
/// References:
/// - Shewhart, W.A. (1931). "Economic Control of Quality of Manufactured Product"
/// - Western Electric (1956). "Statistical Quality Control Handbook"
#[derive(Debug, Clone)]
pub struct ControlChart {
    pub center_line: f64,
    pub upper_control_limit: f64,
    pub lower_control_limit: f64,
    pub upper_warning_limit: f64,
    pub lower_warning_limit: f64,
    pub sigma: f64,
}

impl ControlChart {
    /// Create control chart from data using 3-sigma limits
    pub fn from_data(data: &[f64]) -> Result<Self> {
        if data.len() < 20 {
            return Err(PbjbiError::Statistical(
                "Need at least 20 points for control chart".to_string(),
            ));
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance =
            data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
        let sigma = variance.sqrt();

        Ok(Self {
            center_line: mean,
            upper_control_limit: mean + 3.0 * sigma,
            lower_control_limit: mean - 3.0 * sigma,
            upper_warning_limit: mean + 2.0 * sigma,
            lower_warning_limit: mean - 2.0 * sigma,
            sigma,
        })
    }

    /// Check if a point is within control limits
    pub fn is_in_control(&self, value: f64) -> bool {
        value >= self.lower_control_limit && value <= self.upper_control_limit
    }

    /// Apply Western Electric rules to detect out-of-control conditions
    pub fn apply_western_electric_rules(&self, data: &[f64]) -> Vec<WesternElectricViolation> {
        let mut violations = Vec::new();

        // Rule 1: One point beyond 3σ
        for (i, &value) in data.iter().enumerate() {
            if !self.is_in_control(value) {
                violations.push(WesternElectricViolation {
                    rule: WesternElectricRule::Rule1,
                    index: i,
                    description: format!("Point {} beyond 3σ control limits", i),
                });
            }
        }

        // Rule 2: Two of three consecutive points beyond 2σ on same side
        if data.len() >= 3 {
            for i in 0..data.len() - 2 {
                let window = &data[i..i + 3];
                let above_2sigma = window
                    .iter()
                    .filter(|&&x| x > self.upper_warning_limit)
                    .count();
                let below_2sigma = window
                    .iter()
                    .filter(|&&x| x < self.lower_warning_limit)
                    .count();

                if above_2sigma >= 2 || below_2sigma >= 2 {
                    violations.push(WesternElectricViolation {
                        rule: WesternElectricRule::Rule2,
                        index: i,
                        description: format!("2 of 3 points beyond 2σ starting at {}", i),
                    });
                }
            }
        }

        // Rule 3: Four of five consecutive points beyond 1σ on same side
        if data.len() >= 5 {
            for i in 0..data.len() - 4 {
                let window = &data[i..i + 5];
                let above_1sigma = window
                    .iter()
                    .filter(|&&x| x > self.center_line + self.sigma)
                    .count();
                let below_1sigma = window
                    .iter()
                    .filter(|&&x| x < self.center_line - self.sigma)
                    .count();

                if above_1sigma >= 4 || below_1sigma >= 4 {
                    violations.push(WesternElectricViolation {
                        rule: WesternElectricRule::Rule3,
                        index: i,
                        description: format!("4 of 5 points beyond 1σ starting at {}", i),
                    });
                }
            }
        }

        // Rule 4: Eight consecutive points on same side of center line
        if data.len() >= 8 {
            for i in 0..data.len() - 7 {
                let window = &data[i..i + 8];
                let all_above = window.iter().all(|&x| x > self.center_line);
                let all_below = window.iter().all(|&x| x < self.center_line);

                if all_above || all_below {
                    violations.push(WesternElectricViolation {
                        rule: WesternElectricRule::Rule4,
                        index: i,
                        description: format!("8 consecutive points on same side starting at {}", i),
                    });
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WesternElectricRule {
    Rule1, // One point beyond 3σ
    Rule2, // Two of three beyond 2σ
    Rule3, // Four of five beyond 1σ
    Rule4, // Eight consecutive on same side
}

#[derive(Debug, Clone)]
pub struct WesternElectricViolation {
    pub rule: WesternElectricRule,
    pub index: usize,
    pub description: String,
}

/// Process capability indices for Six Sigma
/// Reference: Montgomery, D.C. (2012). "Introduction to Statistical Quality Control"
#[derive(Debug, Clone)]
pub struct ProcessCapability {
    pub cp: f64,          // Process capability index
    pub cpk: f64,         // Process capability index (adjusted for centering)
    pub sigma_level: f64, // Sigma level (e.g., 6 for Six Sigma)
}

impl ProcessCapability {
    /// Calculate process capability indices
    pub fn calculate(data: &[f64], lower_spec_limit: f64, upper_spec_limit: f64) -> Result<Self> {
        if lower_spec_limit >= upper_spec_limit {
            return Err(PbjbiError::InvalidInput(
                "Lower spec limit must be less than upper spec limit".to_string(),
            ));
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance =
            data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
        let sigma = variance.sqrt();

        if sigma == 0.0 {
            return Err(PbjbiError::Statistical(
                "Process has zero variation".to_string(),
            ));
        }

        // Cp: Process capability
        let cp = (upper_spec_limit - lower_spec_limit) / (6.0 * sigma);

        // Cpk: Process capability adjusted for centering
        let cpu = (upper_spec_limit - mean) / (3.0 * sigma);
        let cpl = (mean - lower_spec_limit) / (3.0 * sigma);
        let cpk = cpu.min(cpl);

        // Sigma level calculation
        let sigma_level = cpk * 3.0;

        Ok(Self {
            cp,
            cpk,
            sigma_level,
        })
    }

    /// Check if process meets Six Sigma standards
    pub fn is_six_sigma(&self) -> bool {
        self.cpk >= 2.0 // Cpk >= 2.0 indicates Six Sigma capability
    }
}

/// Quality report with multiple checks
#[derive(Debug, Clone)]
pub struct QualityReport {
    pub checks: HashMap<String, CheckResult>,
    pub metrics: HashMap<String, f64>,
    pub overall_status: QualityStatus,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualityStatus {
    Pass,
    Warning,
    Fail,
}

impl QualityReport {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            metrics: HashMap::new(),
            overall_status: QualityStatus::Pass,
        }
    }

    pub fn add_check(&mut self, name: impl Into<String>, result: CheckResult) {
        if !result.passed && self.overall_status == QualityStatus::Pass {
            self.overall_status = QualityStatus::Warning;
        }
        self.checks.insert(name.into(), result);
    }

    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }

    pub fn fail(&mut self) {
        self.overall_status = QualityStatus::Fail;
    }
}

impl Default for QualityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Toyota Way implementation for quality management
pub mod toyota_way {
    use super::*;

    /// Jidoka (自働化) - Automatic quality detection
    pub trait Jidoka {
        /// Detect quality issues and stop if found
        fn detect_and_stop(&self, data: &[f64]) -> Result<()>;

        /// Pull andon cord to signal issue
        fn pull_andon_cord(&self, issue: QualityIssue);
    }

    #[derive(Debug, Clone)]
    pub struct QualityIssue {
        pub severity: Severity,
        pub description: String,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Severity {
        Low,
        Medium,
        High,
        Critical,
    }

    /// Kaizen metrics for continuous improvement
    #[derive(Debug, Clone)]
    pub struct KaizenMetrics {
        pub defect_rate_before: f64,
        pub defect_rate_after: f64,
        pub cycle_time_before: f64,
        pub cycle_time_after: f64,
        pub improvement_percentage: f64,
    }

    impl KaizenMetrics {
        pub fn calculate_improvement(before: &ProcessMetrics, after: &ProcessMetrics) -> Self {
            let defect_improvement =
                (before.defect_rate - after.defect_rate) / before.defect_rate * 100.0;
            let cycle_improvement =
                (before.cycle_time - after.cycle_time) / before.cycle_time * 100.0;

            Self {
                defect_rate_before: before.defect_rate,
                defect_rate_after: after.defect_rate,
                cycle_time_before: before.cycle_time,
                cycle_time_after: after.cycle_time,
                improvement_percentage: (defect_improvement + cycle_improvement) / 2.0,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ProcessMetrics {
        pub defect_rate: f64,
        pub cycle_time: f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_chart_creation() {
        let data: Vec<f64> = (0..30).map(|i| 50.0 + (i as f64).sin()).collect();
        let chart = ControlChart::from_data(&data).unwrap();

        assert!(chart.upper_control_limit > chart.center_line);
        assert!(chart.lower_control_limit < chart.center_line);
    }

    #[test]
    fn test_western_electric_rules() {
        let chart = ControlChart {
            center_line: 50.0,
            upper_control_limit: 53.0,
            lower_control_limit: 47.0,
            upper_warning_limit: 52.0,
            lower_warning_limit: 48.0,
            sigma: 1.0,
        };

        // Test Rule 1: Point beyond 3σ
        let data = vec![50.0, 50.5, 54.0, 49.5]; // 54.0 is beyond UCL
        let violations = chart.apply_western_electric_rules(&data);

        assert!(violations
            .iter()
            .any(|v| v.rule == WesternElectricRule::Rule1));
    }

    #[test]
    fn test_process_capability() {
        let data: Vec<f64> = (0..100)
            .map(|_| 50.0 + rand::random::<f64>() * 2.0 - 1.0)
            .collect();

        let capability = ProcessCapability::calculate(&data, 45.0, 55.0).unwrap();

        assert!(capability.cp > 0.0);
        assert!(capability.cpk > 0.0);
    }
}
