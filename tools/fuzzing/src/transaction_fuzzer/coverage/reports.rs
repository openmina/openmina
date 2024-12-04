use super::cov::FileDump;
use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{env, fmt, fs};

//#[derive(Debug, Clone, Serialize)]
pub struct LineCounter {
    pub col_start: usize,
    pub col_end: usize,
    pub count: i64,
}

impl Clone for LineCounter {
    #[coverage(off)]
    fn clone(&self) -> Self {
        Self {
            col_start: self.col_start,
            col_end: self.col_end,
            count: self.count,
        }
    }
}

impl serde::Serialize for LineCounter {
    #[coverage(off)]
    fn serialize<__S>(&self, __serializer: __S) -> serde::__private::Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer,
    {
        let mut __serde_state = match serde::Serializer::serialize_struct(
            __serializer,
            "LineCounter",
            false as usize + 1 + 1 + 1,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(
            &mut __serde_state,
            "col_start",
            &self.col_start,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(
            &mut __serde_state,
            "col_end",
            &self.col_end,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "count", &self.count)
        {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        serde::ser::SerializeStruct::end(__serde_state)
    }
}

impl LineCounter {
    #[coverage(off)]
    fn overlap(&self, rhs: &Self) -> bool {
        self.col_start < rhs.col_end && rhs.col_start < self.col_end
    }

    #[coverage(off)]
    fn contains(&self, rhs: &Self) -> bool {
        self.col_start < rhs.col_start && self.col_end > rhs.col_end
    }

    #[coverage(off)]
    fn merge(&self, rhs: &Self, count: i64) -> Self {
        Self {
            col_start: self.col_start.min(rhs.col_start),
            col_end: self.col_end.max(rhs.col_end),
            count,
        }
    }

    #[coverage(off)]
    fn split3(&self, rhs: &Self) -> Vec<Self> {
        vec![
            Self {
                col_start: self.col_start,
                col_end: rhs.col_start,
                count: self.count,
            },
            Self {
                col_start: rhs.col_start,
                col_end: rhs.col_end,
                count: rhs.count,
            },
            Self {
                col_start: rhs.col_end,
                col_end: self.col_end,
                count: self.count,
            },
        ]
    }

    #[coverage(off)]
    fn split2(&self, rhs: &Self) -> Vec<Self> {
        vec![
            Self {
                col_start: self.col_start,
                col_end: rhs.col_start,
                count: self.count,
            },
            Self {
                col_start: rhs.col_start,
                col_end: rhs.col_end,
                count: rhs.count,
            },
        ]
    }

    #[coverage(off)]
    fn split(&self, rhs: &Self) -> Vec<Self> {
        if self.contains(rhs) {
            self.split3(rhs)
        } else if rhs.contains(self) {
            rhs.split3(self)
        } else if self.col_start < rhs.col_start {
            self.split2(rhs)
        } else {
            rhs.split2(self)
        }
    }

    #[coverage(off)]
    pub fn join(&self, rhs: &Self) -> Option<Vec<Self>> {
        if self.overlap(rhs) {
            if self.count == rhs.count {
                Some(vec![self.merge(rhs, self.count)])
            } else if self.col_start == rhs.col_start && self.col_end == rhs.col_end {
                Some(vec![self.merge(rhs, self.count.max(rhs.count))])
            } else {
                Some(self.split(rhs))
            }
        } else {
            None
        }
    }
}

#[coverage(off)]
fn color_line_counters(line: &str, counters: &[LineCounter]) -> String {
    let mut result = String::new();

    if counters.is_empty() {
        return line.to_string();
    }

    let mut line_color = 43; // light yellow

    for counter in counters.iter() {
        if counter.count > 0 {
            if line_color == 41 {
                line_color = 43;
                break;
            }

            line_color = 42; // light green
        } else {
            if line_color == 42 {
                line_color = 43;
                break;
            }

            line_color = 41; // light red
        }
    }

    result.push_str(&format!("\x1b[1;{}m\x1b[1;37m", line_color));

    for (column, c) in line.chars().enumerate() {
        let counter = counters.iter().find(
            #[coverage(off)]
            |LineCounter {
                 col_start, col_end, ..
             }| *col_start <= column && *col_end >= column,
        );

        if let Some(counter) = counter {
            if column == counter.col_start {
                let color_code: u8 = if counter.count == 0 { 101 } else { 102 };
                result.push_str(&format!("\x1b[1;{}m\x1b[1;{}m", line_color, color_code));
            }
        }

        result.push(c);

        if let Some(counter) = counter {
            // avoid reset colors if there is another counter
            if column == counter.col_end
                && (counter.col_start == counter.col_end
                    || !counters.iter().any(
                        #[coverage(off)]
                        |LineCounter { col_start, .. }| *col_start == column,
                    ))
            {
                result.push_str(&format!("\x1b[1;{}m\x1b[1;37m", line_color));
            }
        }
    }

    result.push_str("\x1b[0m");
    result
}

//#[derive(Debug, Serialize)]
pub struct LineCoverage {
    pub line: String,
    pub counters: Vec<LineCounter>,
}

impl serde::Serialize for LineCoverage {
    #[coverage(off)]
    fn serialize<__S>(&self, __serializer: __S) -> serde::__private::Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer,
    {
        let mut __serde_state = match serde::Serializer::serialize_struct(
            __serializer,
            "LineCoverage",
            false as usize + 1 + 1,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "line", &self.line) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(
            &mut __serde_state,
            "counters",
            &self.counters,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        serde::ser::SerializeStruct::end(__serde_state)
    }
}

impl fmt::Display for LineCoverage {
    #[coverage(off)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", color_line_counters(&self.line, &self.counters))
    }
}

//#[derive(Debug, Serialize)]
pub struct FileCoverage {
    pub filename: String,
    pub lines: Vec<LineCoverage>,
}

impl serde::Serialize for FileCoverage {
    #[coverage(off)]
    fn serialize<__S>(&self, __serializer: __S) -> serde::__private::Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer,
    {
        let mut __serde_state = match serde::Serializer::serialize_struct(
            __serializer,
            "FileCoverage",
            false as usize + 1 + 1,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(
            &mut __serde_state,
            "filename",
            &self.filename,
        ) {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        match serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "lines", &self.lines)
        {
            serde::__private::Ok(__val) => __val,
            serde::__private::Err(__err) => {
                return serde::__private::Err(__err);
            }
        };
        serde::ser::SerializeStruct::end(__serde_state)
    }
}

impl FileCoverage {
    #[coverage(off)]
    pub fn covered_percent(&self) -> usize {
        let mut total = 0;
        let mut covered = 0;

        for line in self.lines.iter() {
            total += line.counters.len();
            covered += line.counters.iter().fold(
                0,
                #[coverage(off)]
                |acc, lc| if lc.count != 0 { acc + 1 } else { acc },
            );
        }

        if total != 0 {
            (covered * 100) / total
        } else {
            0
        }
    }
}

impl fmt::Display for FileCoverage {
    #[coverage(off)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}:", self.filename)?;

        for (i, line) in self.lines.iter().enumerate() {
            writeln!(f, "{:>6}: {}", i, line)?;
        }

        Ok(())
    }
}

//#[derive(Debug)]
pub struct CoverageReport(Vec<FileCoverage>);

impl fmt::Display for CoverageReport {
    #[coverage(off)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for file in self.0.iter() {
            writeln!(f, "{}", file)?;
        }

        Ok(())
    }
}

impl CoverageReport {
    #[coverage(off)]
    pub fn write_files(&self, report_prefix: String) {
        let sources_path = env::var("REPORTS_PATH").unwrap_or("/tmp/".to_string());
        let report_file_vec: Vec<(String, &FileCoverage)> = self
            .0
            .iter()
            .enumerate()
            .map(
                #[coverage(off)]
                |(n, filecov)| (report_prefix.clone() + &n.to_string() + ".json", filecov),
            )
            .collect();

        let report_index: Vec<(String, usize, String)> = report_file_vec
            .iter()
            .map(
                #[coverage(off)]
                |(report_name, filecov)| {
                    (
                        report_name.clone(),
                        filecov.covered_percent(),
                        filecov.filename.clone(),
                    )
                },
            )
            .collect();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(sources_path.clone() + &report_prefix + "index.json")
            .unwrap();

        file.write_all(serde_json::to_string(&report_index).unwrap().as_bytes())
            .unwrap();

        for (report_name, filecov) in report_file_vec.iter() {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(sources_path.clone() + report_name)
                .unwrap();

            file.write_all(serde_json::to_string(filecov).unwrap().as_bytes())
                .unwrap();
        }
    }

    #[coverage(off)]
    fn merge_counters(counter: &LineCounter, counters: &mut Vec<LineCounter>) {
        for idx in 0.. {
            if idx == counters.len() {
                counters.push(counter.clone());
                break;
            }

            if let Some(new_counters) = counters[idx].join(counter) {
                counters.remove(idx);

                for counter in new_counters.iter() {
                    Self::merge_counters(counter, counters);
                }

                break;
            } else if counters[idx].col_end > counter.col_end {
                counters.insert(idx, counter.clone());
                break;
            }
        }
    }

    #[coverage(off)]
    pub fn from_llvm_dump(llvm_dump: &[FileDump]) -> Self {
        let sources_path = env::var("RUST_BUILD_PATH").unwrap_or_else(|_| {
            env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        });
        let mut file_coverage_vec = Vec::new();

        for FileDump {
            filename,
            source_counters_vec,
        } in llvm_dump.iter()
        {
            // Ignore external crates. Might want them back at some point?
            if filename.starts_with("/rustc/")
                || filename.contains("/.cargo/")
                || filename.contains("/.rustup/")
            {
                continue;
            }

            let path = if filename.starts_with("/") {
                filename.clone()
            } else {
                sources_path.clone() + "/" + filename
            };

            let file = File::open(path).unwrap();
            let mut lines: Vec<LineCoverage> = BufReader::new(file)
                .lines()
                .map(
                    #[coverage(off)]
                    |line| LineCoverage {
                        line: line.unwrap(),
                        counters: Vec::new(),
                    },
                )
                .collect();

            for source_counters in source_counters_vec.iter() {
                let mut previous_line = 0;

                for (count, source_range) in source_counters.iter() {
                    let count = *count;
                    let start = previous_line + source_range.delta_line_start - 1;
                    let end = start + source_range.num_lines;
                    //println!("{:?}", source_range);

                    for line in start..=end {
                        if line == lines.len() || lines[line].line.chars().count() == 0 {
                            continue;
                        }

                        let col_start = if line == start {
                            source_range.column_start - 1
                        } else {
                            0
                        };
                        let mut col_end = if line == end {
                            source_range.column_end - 1
                        } else {
                            lines[line].line.chars().count()
                        };

                        // Why can this happen?
                        if col_end < col_start {
                            col_end = col_start;
                        }

                        let line_counter = LineCounter {
                            col_start,
                            col_end,
                            count,
                        };

                        Self::merge_counters(&line_counter, &mut lines[line].counters);
                    }

                    previous_line += source_range.delta_line_start;
                }
            }

            let file_cov = FileCoverage {
                filename: filename.clone(),
                lines,
            };

            file_coverage_vec.push(file_cov);
        }

        Self(file_coverage_vec)
    }

    #[coverage(off)]
    pub fn from_bisect_dump(bisect_dump: &[(String, Vec<i64>, Vec<i64>)]) -> Self {
        let sources_path = env::var("OCAML_BUILD_PATH").unwrap();
        let mut file_coverage_vec = Vec::new();

        for (filename, points, counts) in bisect_dump.iter() {
            let file_contents = fs::read(sources_path.clone() + filename).unwrap();
            let line_offset_vec: Vec<usize> = file_contents
                .iter()
                .enumerate()
                .filter_map(
                    #[coverage(off)]
                    |(i, &x)| if x == b'\n' { Some(i) } else { None },
                )
                .collect();
            let mut lines: Vec<LineCoverage> = line_offset_vec
                .iter()
                .enumerate()
                .map(
                    #[coverage(off)]
                    |(i, &end_pos)| {
                        let start_pos = if i == 0 {
                            0
                        } else {
                            line_offset_vec[i - 1] + 1
                        };
                        LineCoverage {
                            line: String::from_utf8(file_contents[start_pos..end_pos].to_vec())
                                .unwrap(),
                            counters: Vec::new(),
                        }
                    },
                )
                .collect();

            for (point, count) in points
                .iter()
                .zip(counts.iter())
                .filter_map(
                    #[coverage(off)]
                    |(point, count)| {
                        // FIXME: figure out why bisect-ppx is reporting point values such as -1
                        if *point <= 0 {
                            None
                        } else {
                            Some(((*point - 1) as usize, *count))
                        }
                    },
                )
                .sorted_by_key(
                    #[coverage(off)]
                    |(point, _count)| *point,
                )
            {
                let (line_num, _) = line_offset_vec
                    .iter()
                    .enumerate()
                    .find(
                        #[coverage(off)]
                        |(i, &offset)| point < offset || i + 1 == line_offset_vec.len(),
                    )
                    .unwrap();

                let col = if line_num == 0 {
                    point
                } else {
                    point - line_offset_vec[line_num - 1]
                };

                // TODO: find a better way to convert bytes position to char position
                let col_start = String::from_utf8(lines[line_num].line.as_bytes()[..col].to_vec())
                    .unwrap()
                    .chars()
                    .count();

                // It seems there isn't an "end column" in bisect-ppx
                let col_end = col_start;

                lines[line_num].counters.push(LineCounter {
                    col_start,
                    col_end,
                    count,
                })
            }

            file_coverage_vec.push(FileCoverage {
                filename: filename.clone(),
                lines,
            })
        }

        Self(file_coverage_vec)
    }
}
