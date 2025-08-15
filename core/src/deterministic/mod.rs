use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};

/// Deterministic random number generator with reproducible seeds
#[derive(Clone)]
pub struct DeterministicRng {
    seed: u64,
    rng: ChaCha8Rng,
}

impl DeterministicRng {
    /// Create a new deterministic RNG with the given seed
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());

        Self {
            seed,
            rng: ChaCha8Rng::from_seed(seed_bytes),
        }
    }

    /// Get the seed used to initialize this RNG
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Reset the RNG to its initial state
    pub fn reset(&mut self) {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&self.seed.to_le_bytes());
        self.rng = ChaCha8Rng::from_seed(seed_bytes);
    }

    /// Get a mutable reference to the underlying RNG
    pub fn rng_mut(&mut self) -> &mut ChaCha8Rng {
        &mut self.rng
    }
}

/// Checksum for verifying data integrity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checksum {
    hash: String,
}

impl Checksum {
    /// Calculate checksum for arbitrary data
    pub fn calculate<T: serde::Serialize>(data: &T) -> crate::Result<Self> {
        let json = serde_json::to_string(data)
            .map_err(|e| crate::PbjbiError::DeterminismViolation(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hex::encode(hasher.finalize());

        Ok(Self { hash })
    }

    /// Verify that data matches this checksum
    pub fn verify<T: serde::Serialize>(&self, data: &T) -> crate::Result<bool> {
        let calculated = Self::calculate(data)?;
        Ok(calculated.hash == self.hash)
    }

    /// Get the hash string
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// Audit trail for reproducibility
#[derive(Debug, Clone)]
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub input_checksum: Checksum,
    pub output_checksum: Checksum,
    pub seed: Option<u64>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        operation: String,
        input_checksum: Checksum,
        output_checksum: Checksum,
        seed: Option<u64>,
    ) {
        self.entries.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            operation,
            input_checksum,
            output_checksum,
            seed,
        });
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_deterministic_rng_reproducibility() {
        let seed = 42;
        let mut rng1 = DeterministicRng::new(seed);
        let mut rng2 = DeterministicRng::new(seed);

        // Generate some random numbers
        let values1: Vec<u32> = (0..100).map(|_| rng1.rng_mut().gen()).collect();

        let values2: Vec<u32> = (0..100).map(|_| rng2.rng_mut().gen()).collect();

        // Should produce identical sequences
        assert_eq!(values1, values2);
    }

    #[test]
    fn test_checksum_verification() {
        #[derive(serde::Serialize)]
        struct TestData {
            value: i32,
            text: String,
        }

        let data = TestData {
            value: 42,
            text: "test".to_string(),
        };

        let checksum = Checksum::calculate(&data).unwrap();
        assert!(checksum.verify(&data).unwrap());

        let different_data = TestData {
            value: 43,
            text: "test".to_string(),
        };

        assert!(!checksum.verify(&different_data).unwrap());
    }
}
