use derive_builder::Builder;
use thiserror::Error;

use crate::threshold::{Threshold, ThresholdError, ThresholdFor};

use super::measure::PerfStats;

#[derive(Debug, Builder)]
pub struct PerfThresholds {
    pub mean: Threshold<u64>,
}

#[derive(Debug, Error)]
#[error("Performance parameter `{param}`: {error}")]
pub struct PerfThresholdsError {
    error: ThresholdError<u64>,
    param: &'static str,
}

impl ThresholdFor<PerfStats> for PerfThresholds {
    type Error = PerfThresholdsError;

    fn check_threshold(&self, value: &PerfStats, ref_value: &PerfStats) -> Result<(), Self::Error> {
        self.mean
            .check(&value.mean, &ref_value.mean)
            .map_err(|error| PerfThresholdsError {
                error,
                param: "mean",
            })
    }
}
