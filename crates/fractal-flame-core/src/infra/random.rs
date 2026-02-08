use rand::{Rng, rng};

#[derive(Debug, Clone)]
pub enum RangeError {
    InvalidFloatRange { min: f64, max: f64 },
    InvalidIntRange { min: i64, max: i64 },
}

impl std::fmt::Display for RangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeError::InvalidFloatRange { min, max } => {
                write!(f, "invalid float range: min ({min}) > max ({max})")
            }
            RangeError::InvalidIntRange { min, max } => {
                write!(f, "invalid int range: min ({min}) >= max ({max})")
            }
        }
    }
}

impl std::error::Error for RangeError {}

pub fn generate_f64(min: f64, max: f64, include_max: bool) -> Result<f64, RangeError> {
    if min > max {
        return Err(RangeError::InvalidFloatRange { min, max });
    }

    let diff = max - min;

    let include = if include_max { 1u64 } else { 0u64 };
    let upper = 1_000_000_000u64 + include;

    let mut rng = rng();
    let n: u64 = rng.random_range(0..upper);

    let mut value = min + diff * (n as f64 / 1_000_000_000.0);

    if value > max {
        value = max;
    }

    Ok(value)
}

pub fn generate_i32(min: i32, max: i32) -> Result<i32, RangeError> {
    if min >= max {
        return Err(RangeError::InvalidIntRange {
            min: min as i64,
            max: max as i64,
        });
    }

    let mut rng = rng();
    Ok(rng.random_range(min..max))
}
