use std::collections::btree_map::Entry as BTreeMapEntry;
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use regex::Regex;

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
    let crate_dir_name = std::env::var("CARGO_MANIFEST_DIR")?;
    let crate_dir = PathBuf::from(crate_dir_name);
    let node_dir = {
        let mut dir = crate_dir.clone();
        dir.pop();
        dir
    };

    let action_def_re = Regex::new(r"^pub (struct|enum) ([a-zA-Z0-9]*Action)( |\n)\{").unwrap();
    let action_enum_variant_re = Regex::new(r"([a-zA-Z0-9]*)\(([a-zA-Z0-9]*Action)\)").unwrap();

    let mut use_statements: BTreeMap<Vec<String>, Vec<String>> = Default::default();
    use_statements.insert(vec![], vec!["ActionKindGet".to_owned()]);
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

            let Some(matches) = action_def_re.captures(&line) else { continue };
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
                            if line.contains("}") {
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
        if action_defs.len() == 0 {
            return;
        }
        let path = path.strip_prefix(&node_dir).unwrap();
        let use_path = path
            .strip_prefix(crate_dir.file_name().unwrap())
            .unwrap_or(path)
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .filter(|v| v != "src")
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

    let use_deps = [
        "use serde::{Serialize, Deserialize};",
        "use num_enum::TryFromPrimitive;",
    ]
    .join("\n");
    let use_statements = use_statements
        .into_iter()
        .map(|(k, v)| {
            let mut s = "use crate::".to_owned();
            if k.len() != 0 {
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
        .collect::<Vec<_>>()
        .join(",\n    ");

    let action_kind_def = {
        let der =
            "#[derive(Serialize, Deserialize, TryFromPrimitive, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]";
        let repr = "#[repr(u16)]";
        format!("{der}\n{repr}\npub enum ActionKind {{\n    {action_kinds}\n}}")
    };

    let action_kind_get_impls = {
        let mut output = vec![];
        let mut queue = VecDeque::new();
        queue.push_back("Action".to_owned());

        while let Some(action_name) = queue.pop_front() {
            let fn_body = match actions.get(&action_name).unwrap() {
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
        format!("{use_deps}\n\n{use_statements}\n\n{action_kind_def}\n\n{action_kind_get_impls}\n");

    fs::write(crate_dir.join("src/action_kind.rs"), contents)?;

    std::process::Command::new("cargo").arg("fmt").output()?;

    Ok(())
}
