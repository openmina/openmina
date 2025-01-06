use super::cov::FileCounters;
use btreemultimap::BTreeMultiMap;
use std::fmt;

//#[derive(Debug)]
pub struct Stats(BTreeMultiMap<usize, (String, usize, usize)>);

impl Stats {
    // If we are just interested in coverage statistics for some subset of source files
    #[coverage(off)]
    pub fn filter_filenames(&self, filenames: &Vec<&str>) -> Self {
        let mut filtered_map = BTreeMultiMap::new();

        for (k, v) in self.0.iter().filter(
            #[coverage(off)]
            |(_, (filename, _, _))| {
                filenames.iter().any(
                    #[coverage(off)]
                    |name| filename.ends_with(name),
                )
            },
        ) {
            filtered_map.insert(*k, v.clone());
        }

        Self(filtered_map)
    }

    #[coverage(off)]
    pub fn filter_path(&self, path: &str) -> Self {
        let mut filtered_map = BTreeMultiMap::new();

        for (k, v) in self.0.iter().filter(
            #[coverage(off)]
            |(_, (filename, _, _))| !filename.contains(path),
        ) {
            filtered_map.insert(*k, v.clone());
        }

        Self(filtered_map)
    }

    // If we are interested just in top `n` files with more coverage
    #[coverage(off)]
    pub fn filter_top(&self, n: usize) -> Self {
        let mut filtered_map = BTreeMultiMap::new();

        for (k, v) in self.0.clone().iter().rev().take(n) {
            filtered_map.insert(*k, v.clone());
        }

        Self(filtered_map)
    }

    // returns: (coverage percent, total counters, covered counters)
    #[coverage(off)]
    pub fn get_total(&self) -> (usize, usize, usize) {
        let (total, covered) = self.0.iter().fold(
            (0, 0),
            #[coverage(off)]
            |(acc_total, acc_covered), (_, (_, file_total, file_covered))| {
                (acc_total + file_total, acc_covered + file_covered)
            },
        );
        let cov_percent = if total > 0 {
            (covered * 100) / total
        } else {
            0
        };
        (cov_percent, total, covered)
    }

    #[coverage(off)]
    pub fn has_coverage_increased(&self, rhs: &Self) -> bool {
        self.get_total().2 > rhs.get_total().2
    }

    #[coverage(off)]
    pub fn from_file_counters(filecounters: &[FileCounters]) -> Self {
        let mut result = BTreeMultiMap::new();

        for FileCounters {
            filename, counters, ..
        } in filecounters.iter()
        {
            let (total, covered) = counters.iter().fold(
                (0, 0),
                #[coverage(off)]
                |(total, covered), counters| {
                    (
                        total + counters.len(),
                        covered
                            + counters.iter().fold(
                                0,
                                #[coverage(off)]
                                |acc, counter| if *counter != 0 { acc + 1 } else { acc },
                            ),
                    )
                },
            );

            if total != 0 {
                let cov_percent = (covered * 100) / total;
                result.insert(cov_percent, (filename.clone(), total, covered));
            }
        }

        Self(result)
    }

    #[coverage(off)]
    pub fn from_bisect_dump(bisect_dump: &[(String, Vec<i64>, Vec<i64>)]) -> Self {
        let mut result = BTreeMultiMap::new();

        for (filename, _points, counts) in bisect_dump.iter() {
            let covered = counts.iter().fold(
                0,
                #[coverage(off)]
                |acc, counter| if *counter != 0 { acc + 1 } else { acc },
            );
            let total = counts.len();

            if total != 0 {
                let cov_percent = (covered * 100) / total;
                result.insert(cov_percent, (filename.clone(), total, covered));
            }
        }

        Self(result)
    }
}

impl fmt::Display for Stats {
    #[coverage(off)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (percent, (filename, total, covered)) in self.0.iter() {
            writeln!(
                f,
                "{:>3}% {:>4}/{:>4}: {}",
                percent, covered, total, filename
            )?;
        }
        let (percent, total, covered) = self.get_total();
        writeln!(f, "{:>3}% {:>4}/{:>4}: Total", percent, covered, total)
    }
}
