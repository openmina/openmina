// This build script will generate `node/src/action_kind.rs`.
// See the top comment on that file for some context.

use std::collections::btree_map::Entry as BTreeMapEntry;
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use regex::Regex;
use rust_format::Formatter;

fn visit_dirs(dir: &PathBuf, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn is_same_file<P: AsRef<Path>>(file1: P, file2: P) -> Result<bool, std::io::Error> {
    if !file1.as_ref().exists() || !file2.as_ref().exists() {
        return Ok(false);
    }

    let mut f1 = fs::File::open(file1)?;
    let mut f2 = fs::File::open(file2)?;

    // Check if file sizes are different
    if f1.metadata().unwrap().len() != f2.metadata().unwrap().len() {
        return Ok(false);
    }

    let mut b1 = Vec::with_capacity(1024 * 1024);
    f1.read_to_end(&mut b1)?;

    let mut b2 = Vec::with_capacity(b1.len());
    f2.read_to_end(&mut b2)?;

    Ok(b1 == b2)
}

#[derive(Debug)]
enum ActionMeta {
    Struct,
    Enum(Vec<(String, String)>),
}

impl ActionMeta {
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    vergen::EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .all_sysinfo()
        .emit_and_set()?;

    let crate_dir_name = std::env::var("CARGO_MANIFEST_DIR")?;
    let crate_dir = PathBuf::from(crate_dir_name);
    let node_dir = {
        let mut dir = crate_dir.clone();
        dir.pop();
        dir
    };

    let action_def_re = Regex::new(r"^pub (struct|enum) ([a-zA-Z0-9]*Action)( |\n)\{").unwrap();
    let action_enum_variant_re =
        Regex::new(r"([a-zA-Z0-9]*)\(\n? *([a-zA-Z0-9]*Action),?\n? *\)").unwrap();

    let mut use_statements: BTreeMap<Vec<String>, Vec<String>> = Default::default();
    use_statements.insert(vec![], vec!["ActionKindGet".to_owned()]);
    // `BTreeMap` will ensure that the order is deterministic across runs.
    let mut actions: BTreeMap<String, ActionMeta> = Default::default();

    visit_dirs(&node_dir, &mut |file| {
        let mut path = file.path();
        let path_str = path.to_str().unwrap();
        if !path_str.ends_with("_actions.rs") && !path_str.ends_with("action.rs") {
            return;
        }

        let file = fs::File::open(&path).unwrap();
        let reader = BufReader::new(file);

        let mut lines = reader.lines();
        let mut action_defs: Vec<String> = vec![];

        loop {
            let Some(line) = lines.next() else { break };
            let line = line.unwrap();

            let Some(matches) = action_def_re.captures(&line) else {
                continue;
            };
            match &matches[1] {
                "struct" => {
                    if let Some(action_name) = matches.get(2) {
                        let action_name = action_name.as_str().to_owned();
                        actions.insert(action_name.clone(), ActionMeta::Struct);
                        action_defs.push(action_name);
                    }
                }
                "enum" => {
                    if let Some(action_name) = matches.get(2) {
                        let action_name = action_name.as_str().to_owned();
                        let mut variant_lines = vec![];
                        loop {
                            let Some(line) = lines.next() else { break };
                            let line = line.unwrap();
                            if line.contains('}') {
                                break;
                            }
                            variant_lines.push(line);
                        }
                        let variants_str = variant_lines.join("");

                        let variants = action_enum_variant_re
                            .captures_iter(&variants_str)
                            .map(|matches| (matches[1].to_owned(), matches[2].to_owned()))
                            .collect();
                        actions.insert(action_name.clone(), ActionMeta::Enum(variants));
                        action_defs.push(action_name);
                    }
                }
                _ => continue,
            }
        }

        path.pop();
        if action_defs.is_empty() {
            return;
        }
        let path = path.strip_prefix(&node_dir).unwrap();
        let use_path = path
            .strip_prefix(crate_dir.file_name().unwrap())
            .unwrap_or(path)
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .filter(|v| v != "src" && v != "node")
            .collect::<Vec<_>>();

        match use_statements.entry(use_path) {
            BTreeMapEntry::Vacant(v) => {
                v.insert(action_defs);
            }
            BTreeMapEntry::Occupied(mut v) => {
                v.get_mut().extend(action_defs);
            }
        }
    })?;

    let top_comment = [
        "// DO NOT EDIT. GENERATED BY `node/build.rs`",
        "//",
        "// This file includes the [ActionKindGet] trait implementation for all action variants.",
        "// It also defines the [ActionKind] enum that consolidates all action types.",
        "//",
        "// Why? As a substitute of the derive macro provided by the [enum-kinds](https://crates.io/crates/enum-kinds).",
        "//",
        "// This arrangement helps eliminate macro overhead while also enabling us to aggregate multiple action types",
        "// into an single [ActionKind] enum.",
        "// That enables uncoupling through the partitioning of actions into multiple types that get combined by a",
        "// top-level [Action] type in a way that helps the compiler avoid the recompilation of all the actions",
        "// related code for every single change."
    ].join("\n");

    let use_deps = [
        "use serde::{Serialize, Deserialize};",
        "use num_enum::TryFromPrimitive;",
    ]
    .join("\n");
    let use_statements = use_statements
        .into_iter()
        .map(|(k, v)| {
            let mut s = "use crate::".to_owned();
            if !k.is_empty() {
                s += &k.join("::");
                s += "::";
            }
            s += &format!("{{{}}};", v.join(", "));
            s
        })
        .collect::<Vec<_>>()
        .join("\n");

    let action_kinds_iter = actions
        .iter()
        .filter(|(_, meta)| meta.is_struct())
        // Remove suffix `Action` from action name.
        .map(|(name, _)| name[..(name.len() - 6)].to_string());
    let action_kinds = std::iter::once("None".to_owned())
        .chain(action_kinds_iter)
        .collect::<Vec<_>>();

    let action_kind_def = {
        let comment = "/// Unified kind enum for all action types";
        let der =
            "#[derive(Serialize, Deserialize, TryFromPrimitive, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]";
        let repr = "#[repr(u16)]";
        let impl_ = format!(
            "impl ActionKind {{\n    pub const COUNT: u16 = {};\n}}",
            action_kinds.len()
        );
        let impl_display = format!(
            "impl std::fmt::Display for ActionKind {{{}{}{}{}",
            "\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {",
            "\n        write!(f, \"{self:?}\")",
            "\n    }",
            "}",
        );
        let action_kinds = action_kinds.join(",\n    ");
        format!("{comment}\n{der}\n{repr}\npub enum ActionKind {{\n    {action_kinds}\n}}\n\n{impl_}\n\n{impl_display}")
    };

    let action_kind_get_impls = {
        let mut output = vec![];
        let mut queue = VecDeque::new();
        queue.push_back("Action".to_owned());

        while let Some(action_name) = queue.pop_front() {
            let fn_body = match actions.get(dbg!(&action_name)).unwrap() {
                ActionMeta::Struct => {
                    let action_kind = &action_name[..(action_name.len() - 6)];
                    format!("ActionKind::{action_kind}")
                }
                ActionMeta::Enum(variants) => {
                    for (_, a) in variants {
                        queue.push_back(a.clone());
                    }

                    let variants_iter = variants
                        .iter()
                        .map(|(v, _)| format!("            Self::{v}(a) => a.kind(),"));
                    std::iter::once("match self {".to_owned())
                        .chain(variants_iter)
                        .chain(std::iter::once("        }".to_owned()))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };
            let fn_impl = format!("    fn kind(&self) -> ActionKind {{\n        {fn_body}\n    }}");
            let trait_impl = format!("impl ActionKindGet for {action_name} {{\n{fn_impl}\n}}");
            output.push(trait_impl);
        }

        output.join("\n\n")
    };

    let contents =
        format!("{top_comment}\n\n{use_deps}\n\n{use_statements}\n\n{action_kind_def}\n\n{action_kind_get_impls}\n");

    let tmp_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("action_kind.rs");
    fs::write(&tmp_path, contents)?;
    rust_format::RustFmt::default()
        .format_file(&tmp_path)
        .expect("failed to format generated file");

    let path = crate_dir.join("src/action_kind.rs");

    if !is_same_file(&tmp_path, &path)? {
        fs::rename(tmp_path, path)?;
    }

    Ok(())
}
