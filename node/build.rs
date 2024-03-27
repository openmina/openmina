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

#[derive(Debug, Clone)]
enum EnumElement {
    InlineFields {
        tag: String,
        action_kind: String,
        has_fields: bool,
    },
    Nested((String, String)),
}

#[derive(Debug)]
enum ActionMeta {
    Struct,
    Enum(Vec<EnumElement>),
}

impl ActionMeta {
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct)
    }

    pub fn is_inlined(&self) -> bool {
        if let Self::Enum(fields) = self {
            fields
                .iter()
                .any(|field| matches!(field, EnumElement::InlineFields { .. }))
        } else {
            false
        }
    }

    pub fn action_kinds(&self) -> Vec<String> {
        match self {
            Self::Struct => vec![],
            Self::Enum(elts) => {
                let mut action_kinds = elts
                    .iter()
                    .filter_map(|elt| match elt {
                        EnumElement::Nested(_) => None,
                        EnumElement::InlineFields { action_kind, .. } => Some(action_kind.clone()),
                    })
                    .collect::<Vec<_>>();
                action_kinds.sort();
                action_kinds
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    vergen::EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .emit_and_set()?;

    let crate_dir_name = std::env::var("CARGO_MANIFEST_DIR")?;
    let crate_dir = PathBuf::from(crate_dir_name);
    let node_dir = {
        let mut dir = crate_dir.clone();
        dir.pop();
        dir
    };

    let action_def_re = Regex::new(r"^pub (struct|enum) ([a-zA-Z0-9]*Action)( |\n)\{").unwrap();
    let action_enum_variant_nested_re =
        Regex::new(r"([a-zA-Z0-9]*)\(\n? *([a-zA-Z0-9]*Action),?\n? *\)").unwrap();
    let action_enum_variant_inline_re =
        Regex::new(r"(?m)^\s*([A-Z][a-zA-Z0-9]+)(\s*\{[^}]*\})?,").unwrap();

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
                        // Without 'Action' suffix
                        let action_name_base = action_name[..(action_name.len() - 6)].to_string();
                        let mut variant_lines = vec![];
                        loop {
                            let Some(line) = lines.next() else { break };
                            let line = line.unwrap();
                            if line.ends_with('}') {
                                break;
                            }
                            variant_lines.push(line);
                        }
                        let variants_str = variant_lines.join("\n");

                        let variants_nested = action_enum_variant_nested_re
                            .captures_iter(&variants_str)
                            .map(|matches| {
                                EnumElement::Nested((matches[1].to_owned(), matches[2].to_owned()))
                            })
                            .collect::<Vec<_>>();
                        let variants_inlined = action_enum_variant_inline_re
                            .captures_iter(&variants_str)
                            .filter_map(|matches| {
                                let tag = matches[1].to_owned();
                                if tag.ends_with("Action") {
                                    None
                                } else {
                                    let action_kind = format!("{action_name_base}{tag}");
                                    Some(EnumElement::InlineFields {
                                        tag,
                                        action_kind,
                                        has_fields: matches.get(2).is_some(),
                                    })
                                }
                            })
                            .collect::<Vec<_>>();
                        let variants = variants_nested
                            .iter()
                            .chain(variants_inlined.iter())
                            .cloned()
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
            .iter()
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
        .filter(|(_, meta)| meta.is_struct() || meta.is_inlined())
        .flat_map(|(name, meta)| {
            if meta.is_inlined() {
                meta.action_kinds()
            } else {
                // Remove suffix `Action` from action name.
                vec![name[..(name.len() - 6)].to_string()]
            }
        });
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
                    for elt in variants {
                        if let EnumElement::Nested((_, a)) = elt {
                            queue.push_back(a.clone());
                        }
                    }

                    let variants_iter = variants.iter().map(|elt| match elt {
                        EnumElement::Nested((v, _)) => {
                            format!("            Self::{v}(a) => a.kind(),")
                        }
                        EnumElement::InlineFields {
                            tag,
                            action_kind,
                            has_fields: true,
                        } => {
                            format!(
                                "            Self::{tag} {{ .. }} => ActionKind::{action_kind},"
                            )
                        }
                        EnumElement::InlineFields {
                            tag,
                            action_kind,
                            has_fields: false,
                        } => {
                            format!("            Self::{tag} => ActionKind::{action_kind},")
                        }
                    });
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
