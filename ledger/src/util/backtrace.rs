use backtrace::Backtrace;
use std::fmt::Write;

/// Make a backtrace without OCaml dependencies
/// Those are very noisy and are not meaningful for our case
/// Only Rust and Mina codebase symbols should remain.
pub fn short_backtrace() -> String {
    let bt = Backtrace::new();

    let mut s = String::with_capacity(2000);

    for (index, (filename, name, line)) in bt
        .frames()
        .iter()
        .flat_map(|frame| frame.symbols())
        .map(|sym| (sym.filename(), sym.name(), sym.lineno()))
        .enumerate()
        .filter(|(_, (filename, name, _))| {
            return !(filename
                .map(|filename| {
                    filename.ends_with("ledger/src/util/backtrace.rs")
                        || filename
                            .components()
                            .any(|comp| comp.as_os_str() == "_opam")
                })
                .unwrap_or(false)
                || name
                    .as_ref()
                    .and_then(|n| n.as_str())
                    .map(|name| {
                        name.starts_with("camlAsync_")
                            || name.starts_with("camlBase__")
                            || name.starts_with("camlCore__")
                            || name.starts_with("camlCamlinternalLazy__")
                            || name.starts_with("camlO1trace__") // This belongs to the mina repo, but useless to us
                    })
                    .unwrap_or(false));
        })
        .take_while(|(_, (_, name, _))| {
            name.as_ref().and_then(|n| n.as_str()) != Some("caml_program")
        })
    {
        match (name, filename, line) {
            (Some(name), None, None) => writeln!(&mut s, " {:>3} - {}", index, name).unwrap(),
            (Some(name), Some(filename), None) => {
                writeln!(&mut s, " {:>3} - {:80} ({:?})", index, name, filename).unwrap();
            }
            (Some(name), Some(filename), Some(line)) => {
                writeln!(
                    &mut s,
                    " {:>3} - {:80} ({:?}:{:?})",
                    index, name, filename, line
                )
                .unwrap();
            }
            _ => {}
        }
    }

    s.pop(); // Remove last newline
    s
}
