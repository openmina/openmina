use std::{
    env,
    fs::{self, File},
    io,
    process::{self, Child, Command, Stdio},
    time::Duration,
};

use node::core::thread;

fn main() {
    let mut debugger = Command::new("bpf-recorder")
        .envs([("RUST_LOG", "info"), ("SERVER_PORT", "8000")])
        .stderr(Stdio::piped())
        .spawn()
        .expect("cannot run debugger");
    thread::sleep(Duration::from_secs(2));
    let mut test = Command::new(env::args().nth(1).unwrap())
        .args(env::args().skip(2))
        .envs(env::vars())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cannot spawn the test");
    {
        let mut stdout = test.stdout.take().expect("must be stdout");
        let mut log = File::create("test.log").expect("failed to create test log file");
        thread::spawn(move || {
            io::copy(&mut stdout, &mut log).expect("failed to store test log");
        });
    }
    {
        let mut stderr = debugger.stderr.take().expect("must be stderr");
        let mut log = File::create("debugger.log").expect("failed to create debugger log file");
        thread::spawn(move || {
            io::copy(&mut stderr, &mut log).expect("failed to store debugger log");
        });
    }
    let test_status = test.wait().expect("cannot run the test");
    kill(debugger);
    fs::remove_dir_all("target/db").unwrap_or_default();
    if !test_status.success() {
        println!("test failed, log:");
        let mut log = File::open("test.log").expect("failed to open test log file");
        io::copy(&mut log, &mut io::stdout()).expect("failed to print test log");
        // println!("debugger log:");
        // let mut log = File::open("debugger.log").expect("failed to open test debugger file");
        // io::copy(&mut log, &mut io::stdout()).expect("failed to print debugger log");
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
