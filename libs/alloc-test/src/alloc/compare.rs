use derive_builder::Builder;
use thiserror::Error;

use super::measure::MemoryStats;
use crate::threshold::{Threshold, ThresholdError, ThresholdFor};

/// Limits for each allocation statistics parameter.
#[derive(Debug, Builder)]
pub struct AllocThresholds {
    #[builder(default)]
    pub current: Threshold<usize>,
    #[builder(default)]
    pub peak: Threshold<usize>,
    #[builder(default)]
    pub total_size: Threshold<usize>,
    #[builder(default)]
    pub total_num: Threshold<usize>,
    #[builder(default)]
    pub reallocs: Threshold<usize>,
}

#[derive(Debug, Error)]
#[error("Allocation parameter `{param}`: {error}")]
pub struct AllocThresholdsError {
    error: ThresholdError<usize>,
    param: &'static str,
}

macro_rules! check {
    ($f:ident, $l:expr, $v:expr, $r:expr) => {
        $l.$f
            .check(&$v.$f, &$r.$f)
            .map_err(|e| AllocThresholdsError {
                error: e,
                param: stringify!($f),
            })
    };
}

impl ThresholdFor<MemoryStats> for AllocThresholds {
    type Error = AllocThresholdsError;

    fn check_threshold(
        &self,
        value: &MemoryStats,
        ref_value: &MemoryStats,
    ) -> Result<(), Self::Error> {
        self.check(value, ref_value)
    }
}

impl AllocThresholds {
    pub fn check(
        &self,
        value: &MemoryStats,
        ref_value: &MemoryStats,
    ) -> Result<(), AllocThresholdsError> {
        check!(current, self, value, ref_value)?;
        check!(peak, self, value, ref_value)?;
        check!(total_size, self, value, ref_value)?;
        check!(total_num, self, value, ref_value)?;
        check!(reallocs, self, value, ref_value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits() {
        let rs = MemoryStats {
            current: 100,
            peak: 1000,
            total_size: 2000,
            total_num: 100,
            reallocs: 0,
        };
        let vs = MemoryStats {
            current: 110,
            peak: 1100,
            total_size: 2200,
            total_num: 110,
            reallocs: 1,
        };

        let ls = AllocThresholdsBuilder::default().build().unwrap();
        assert!(ls.check_threshold(&rs, &rs).is_ok());
        assert!(ls.check_threshold(&vs, &rs).is_ok());

        let ls = AllocThresholdsBuilder::default()
            .current(Threshold::Cap(1))
            .build()
            .unwrap();
        let r = ls.check(&vs, &rs);
        assert!(r.unwrap_err().param == "current");

        let ls = AllocThresholdsBuilder::default()
            .reallocs(Threshold::Cap(0))
            .build()
            .unwrap();
        let r = ls.check(&vs, &rs);
        assert!(r.unwrap_err().param == "reallocs");
    }
}
