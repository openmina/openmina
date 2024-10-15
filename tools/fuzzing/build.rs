use std::process::Command;

fn main() {
    let output = Command::new("rustc")
        .args(&["-vV"])
        .output()
        .expect("Failed to execute rustc");

    let stdout = String::from_utf8(output.stdout).unwrap();

    if stdout.contains("nightly") {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
