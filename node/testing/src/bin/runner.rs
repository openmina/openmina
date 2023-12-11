use std::{
    env::args,
    fs::{self, File},
    io,
    process::{self, Child, Command, Stdio},
    thread,
};

fn main() {
    let debugger = Command::new("bpf-recorder")
        .envs([("RUST_LOG", "none"), ("SERVER_PORT", "8000")])
        .spawn()
        .expect("cannot run debugger");
    let mut test = Command::new(args().next().unwrap())
        .args(args().skip(1))
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cannot spawn the test");
    let mut stdout = test.stdout.take().expect("must be stdout");
    let mut test_log = File::create("test.log").expect("failed to create test log file");
    thread::spawn(move || {
        io::copy(&mut stdout, &mut test_log).expect("failed to store test log");
    });
    let test_status = test.wait().expect("cannot run the test");
    kill(debugger);
    fs::remove_dir_all("target/db").unwrap_or_default();
    if !test_status.success() {
        process::exit(test_status.code().unwrap_or(-1));
    }
}

fn kill(mut subprocess: Child) {
    use nix::{
        sys::signal::{self, Signal},
        unistd::Pid,
    };

    if let Err(err) = signal::kill(Pid::from_raw(subprocess.id() as i32), Signal::SIGINT) {
        eprintln!("error sending ctrl+c to Network debugger: {err}");
    }
    match subprocess.try_wait() {
        Err(err) => {
            eprintln!("error getting status from Network debugger: {err}");
        }
        Ok(None) => {
            eprintln!("error getting status from Network debugger");
        }
        Ok(Some(status)) => {
            eprintln!("network debugger {status}");
        }
    }
}
