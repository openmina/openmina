use std::{
    env,
    fmt::{Debug, Display},
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use num::{bigint::ToBigInt, rational::Ratio, Integer, ToPrimitive};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, derive_more::Display)]
pub enum Threshold<T: Display + Integer + ToBigInt + ToPrimitive + Clone> {
    #[default]
    None,
    #[display(fmt = "{_0}")]
    Cap(T),
    #[display(fmt = "{}", "_0.to_f64().unwrap()")]
    Ratio(Ratio<T>),
}

impl<T> Threshold<T>
where
    T: Clone + Integer + ToBigInt + ToPrimitive + Display,
{
    pub fn cap(cap: T) -> Self {
        Threshold::Cap(cap)
    }

    pub fn ratio(numer: T, denom: T) -> Self {
        Threshold::Ratio(Ratio::new(numer, denom))
    }
}

#[derive(Debug, Error)]
#[error("{value} exceeds {ref_value} by more than {limit}")]
pub struct ThresholdError<T: Display + Integer + ToBigInt + ToPrimitive + Clone> {
    limit: Threshold<T>,
    value: T,
    ref_value: T,
}

impl<T> Threshold<T>
where
    T: Clone + Integer + ToBigInt + ToPrimitive + Display,
{
    fn check_cap(cap: &T, value: &T, ref_value: &T) -> bool {
        value.clone() <= ref_value.clone() + cap.clone()
    }

    fn check_ratio(ratio: &Ratio<T>, value: &T, ref_value: &T) -> bool {
        value.clone() <= ref_value.clone()
            || Ratio::new(value.clone() - ref_value.clone(), ref_value.clone()) <= *ratio
    }

    pub fn check(&self, value: &T, ref_value: &T) -> Result<(), ThresholdError<T>> {
        match self {
            Threshold::Cap(cap) if !Self::check_cap(cap, value, ref_value) => Err(ThresholdError {
                limit: self.clone(),
                value: value.clone(),
                ref_value: ref_value.clone(),
            }),
            Threshold::Ratio(ratio) if !Self::check_ratio(ratio, value, ref_value) => {
                Err(ThresholdError {
                    limit: self.clone(),
                    value: value.clone(),
                    ref_value: ref_value.clone(),
                })
            }
            _ => Ok(()),
        }
    }
}

pub trait ThresholdFor<T> {
    type Error;
    fn check_threshold(&self, value: &T, ref_value: &T) -> Result<(), Self::Error>;
}

impl<T> ThresholdFor<T> for Threshold<T>
where
    T: Clone + Integer + ToBigInt + ToPrimitive + Display,
{
    type Error = ThresholdError<T>;

    fn check_threshold(&self, value: &T, ref_value: &T) -> Result<(), Self::Error> {
        self.check(value, ref_value)
    }
}

pub fn check_threshold<F: Fn() -> T, H: ThresholdFor<T>, T>(
    f: F,
    ref_value: &T,
    threshold: H,
) -> Result<T, H::Error> {
    let value = f();
    threshold.check_threshold(&value, ref_value)?;
    Ok(value)
}

#[derive(Debug, thiserror::Error)]
pub enum CheckThresholdError<T: Debug + Display> {
    #[error("regression detected: {_0}")]
    Regression(T),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] toml::de::Error),
}

pub fn check_threshold_with_io<F, H, T>(
    f: F,
    baseline: &Path,
    load_prev: bool,
    strict_compare: bool,
    save_new: bool,
    threshold: &H,
) -> Result<T, CheckThresholdError<H::Error>>
where
    F: Fn() -> T,
    H: ThresholdFor<T>,
    T: Debug + Serialize + DeserializeOwned,
    <H as ThresholdFor<T>>::Error: Debug + Display,
{
    let value = f();
    if load_prev {
        match fs::read_to_string(baseline) {
            Ok(content) => {
                let ref_value = toml::from_str::<T>(&content)?;
                threshold
                    .check_threshold(&value, &ref_value)
                    .map_err(CheckThresholdError::Regression)?;
            }
            Err(e) if !strict_compare && e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }

    if save_new {
        // shouldn't panic unless `MemoryStats` contains unsupported data types
        let stats = toml::to_string(&value).unwrap_or_else(|e| {
            unreachable!("cannot unparse stats into toml: {e}\ndata: {value:#?}")
        });

        match baseline.parent() {
            None => unreachable!("cannot gen parent of `{baseline:?}`"),
            Some(p) if !p.exists() => fs::create_dir_all(p)?,
            _ => {}
        }

        fs::write(baseline, stats.as_bytes())?;
    }
    Ok(value)
}

pub fn check_threshold_with_str<'a, F, H, T>(
    f: F,
    baseline: &'a str,
    threshold: &H,
) -> Result<T, CheckThresholdError<H::Error>>
where
    F: Fn() -> T,
    H: ThresholdFor<T>,
    T: Serialize + Deserialize<'a>,
    <H as ThresholdFor<T>>::Error: Debug + Display,
{
    let ref_value = toml::from_str(&baseline)?;
    let value = f();
    threshold
        .check_threshold(&value, &ref_value)
        .map_err(CheckThresholdError::Regression)?;
    Ok(value)
}

#[derive(Debug, Parser)]
struct MemBenchArgs {
    #[arg(short, long, value_name = "DIR", env)]
    load_baseline: Option<PathBuf>,
    #[arg(short, long, value_name = "DIR")]
    save_baseline: Option<PathBuf>,
    #[arg(short, long)]
    discard_baseline: bool,
}

fn parse_args() -> MemBenchArgs {
    let (test_n, exact) =
        env::args()
            .skip(1)
            .take_while(|a| a != "--")
            .fold((0, false), |(n, e), a| match a.as_str() {
                "--exact" => (n, true),
                _ if !a.starts_with("-") => (n + 1, e),
                _ => (n, e),
            });
    if test_n != 1 {
        panic!("specify exactly one test to run");
    }
    if !exact {
        panic!("make sure only one test is executed by adding `--exact` parameter")
    }
    // TODO replace argv[0] with something sensible
    MemBenchArgs::parse_from(env::args().skip_while(|a| a != "--"))
}

/// Returns the Cargo target directory, possibly calling `cargo metadata` to
/// figure it out.
fn cargo_target_directory() -> Option<PathBuf> {
    #[derive(Deserialize, Debug)]
    struct Metadata {
        target_directory: PathBuf,
    }

    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let output = Command::new(env::var_os("CARGO")?)
                .args(&["metadata", "--format-version", "1"])
                .output()
                .ok()?;
            let metadata: Metadata = serde_json::from_slice(&output.stdout).ok()?;
            Some(metadata.target_directory)
        })
}

fn default_dir(dir: &str) -> PathBuf {
    cargo_target_directory()
        .unwrap_or_else(PathBuf::new)
        .join(dir)
}

const EXT: &str = "toml";

pub fn check_threshold_with_args<F, H, T>(
    f: F,
    dir: &str,
    id: &str,
    threshold: &H,
) -> Result<T, CheckThresholdError<H::Error>>
where
    F: Fn() -> T,
    H: ThresholdFor<T>,
    T: Debug + Serialize + DeserializeOwned,
    <H as ThresholdFor<T>>::Error: Debug + Display,
{
    let args = parse_args();
    let (baseline, load_prev, strict_compare, save_new) = match args {
        MemBenchArgs {
            load_baseline: Some(baseline),
            save_baseline: None,
            discard_baseline: false,
        } => (baseline, true, true, false),
        MemBenchArgs {
            load_baseline: None,
            save_baseline: Some(baseline),
            discard_baseline: false,
        } => (baseline, false, false, true),
        MemBenchArgs {
            load_baseline: None,
            save_baseline: None,
            discard_baseline,
        } => (default_dir(dir), false, false, !discard_baseline),
        _ => panic!("At most one option should be specified"),
    };

    let baseline = baseline.join(id).with_extension(EXT);
    check_threshold_with_io(f, &baseline, load_prev, strict_compare, save_new, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_cap() {
        let l = Threshold::cap(10_u32);
        let r = 100_u32;
        assert!(l.check(&0, &r).is_ok());
        assert!(l.check(&100, &r).is_ok());
        assert!(l.check(&110, &r).is_ok());
        assert!(l.check(&111, &r).is_err());

        println!("{}", l.check(&111, &r).unwrap_err());
    }

    #[test]
    fn limit_ratio() {
        let l = Threshold::ratio(1, 10);
        let r = 100_u32;
        assert!(l.check(&0, &r).is_ok());
        assert!(l.check(&100, &r).is_ok());
        assert!(l.check(&110, &r).is_ok());
        assert!(l.check(&111, &r).is_err());

        println!("{}", l.check(&111, &r).unwrap_err());
    }
}
