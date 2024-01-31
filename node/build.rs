// This build script will generate `node/src/action_kind.rs`.
// See the top comment on that file for some context.

use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, DirEntry};
use std::io::Read;
use std::path::{Path, PathBuf};

use rust_format::Formatter;
use syn::{ItemEnum, ItemStruct};

fn visit_dirs<F: FnMut(&DirEntry) -> anyhow::Result<()>>(
    dir: &PathBuf,
    cb: &mut F,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

fn trim_action_name(s: &str) -> &str {
    s.trim_end_matches("Action")
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
    EnumStruct(Vec<String>),
}

impl ActionMeta {
    pub fn is_leaf(&self) -> bool {
        matches!(self, ActionMeta::Struct)
    }
}

fn main() -> anyhow::Result<()> {
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

    let mut use_statements: BTreeMap<Vec<String>, Vec<String>> = Default::default();
    use_statements.insert(vec![], vec!["ActionKindGet".to_owned()]);
    // `BTreeMap` will ensure that the order is deterministic across runs.
    let mut actions: BTreeMap<String, ActionMeta> = Default::default();

    visit_dirs(&node_dir, &mut |file| {
        let mut path = file.path();
        let path_str = path.to_str().unwrap();
        if !path_str.ends_with("_actions.rs") && !path_str.ends_with("action.rs") {
            return Ok(());
        }
        let mut action_defs: Vec<String> = vec![];

        {
            let file = std::fs::read_to_string(file.path())?;
            let file = syn::parse_str::<syn::File>(&file)?;

            for item in file.items {
                match item {
                    syn::Item::Enum(ItemEnum {
                        ident, variants, ..
                    }) if ident.to_string().ends_with("Action") => {
                        match variants.first().map(|v| &v.fields) {
                            None | Some(syn::Fields::Unit) => break,
                            Some(syn::Fields::Unnamed(syn::FieldsUnnamed { .. })) => {
                                let variants = variants
                                    .into_iter()
                                    .map(|variant| {
                                        let v1 = variant.ident.to_string();
                                        let syn::Fields::Unnamed(syn::FieldsUnnamed {
                                            unnamed,
                                            ..
                                        }) = variant.fields
                                        else {
                                            anyhow::bail!("unexpected variant {v1}");
                                        };
                                        let mut unnamed_it = unnamed.into_iter();
                                        let Some(inner) = unnamed_it.next() else {
                                            anyhow::bail!("single item tuple expected: {v1}");
                                        };
                                        if unnamed_it.next().is_some() {
                                            anyhow::bail!("single item tuple expected: {v1}");
                                        }
                                        let syn::Type::Path(syn::TypePath { path, .. }) = inner.ty
                                        else {
                                            anyhow::bail!("single item tuple expected: {v1}");
                                        };
                                        let mut items = path.segments.into_iter();
                                        let Some(last) = items.next_back() else {
                                            anyhow::bail!("empty path: {v1}");
                                        };
                                        Ok((v1, last.ident.to_string()))
                                    })
                                    .collect::<Result<_, _>>()?;
                                actions.insert(ident.to_string(), ActionMeta::Enum(variants));
                                action_defs.push(ident.to_string());
                            }
                            Some(syn::Fields::Named(syn::FieldsNamed { .. })) => {
                                let variants = variants
                                    .into_iter()
                                    .map(|variant| variant.ident.to_string())
                                    .collect::<Vec<_>>();
                                let name = trim_action_name(&ident.to_string()).to_string();
                                for v in &variants {
                                    let action_kind = format!("{name}{v}Action");
                                    actions.insert(action_kind.clone(), ActionMeta::Struct);
                                }
                                actions.insert(ident.to_string(), ActionMeta::EnumStruct(variants));
                                action_defs.push(ident.to_string());
                            }
                        }
                    }
                    syn::Item::Struct(ItemStruct { ident, .. })
                        if ident.to_string().ends_with("Action") =>
                    {
                        actions.insert(ident.to_string(), ActionMeta::Struct);
                        action_defs.push(ident.to_string());
                    }
                    _ => {}
                }
            }
        }

        path.pop();
        if action_defs.is_empty() {
            return Ok(());
        }
        let path = path.strip_prefix(&node_dir).unwrap();
        let use_path = path
            .strip_prefix(crate_dir.file_name().unwrap())
            .unwrap_or(path)
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .filter(|v| v != "src" && v != "node")
            .collect::<Vec<_>>();

        use_statements.entry(use_path).or_default().extend(action_defs);
        Ok(())
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
        .filter(|(_, meta)| meta.is_leaf())
        // Remove suffix `Action` from action name.
        .map(|(name, _)| trim_action_name(name).to_string());
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
                    let action_kind = trim_action_name(&action_name);
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
                ActionMeta::EnumStruct(variants) => {
                    let variants_iter = variants
                        .iter()
                        .map(|v| format!("            Self::{v} {{ .. }} => ActionKind::{}{},", trim_action_name(&action_name), v));
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
