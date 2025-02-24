use backtrace::Backtrace;
use std::panic::PanicHookInfo;

#[cfg(not(target_arch = "wasm32"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod commands;
use clap::Parser;

mod exit_with_error;
pub use exit_with_error::exit_with_error;

#[cfg(feature = "unsafe-signal-handlers")]
mod unsafe_signal_handlers {
    use nix::libc;

    extern "C" fn handle_sigsegv(_signal: libc::c_int) {
        eprintln!("########### SIGSEGV #############");
        node::recorder::Recorder::graceful_shutdown();
        std::process::exit(1);
    }

    pub fn setup() {
        let stack_t = libc::stack_t {
            ss_sp: {
                let stack = Box::<[u8; libc::SIGSTKSZ]>::new([0; libc::SIGSTKSZ]);
                Box::into_raw(stack) as *mut _
            },
            ss_flags: 0,
            ss_size: libc::SIGSTKSZ,
        };

        let res = unsafe { libc::sigaltstack(&stack_t as *const _, std::ptr::null_mut()) };
        assert_eq!(res, 0);

        let action = libc::sigaction {
            sa_sigaction: handle_sigsegv as _,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: libc::SA_ONSTACK,
            sa_restorer: None,
        };
        let res = unsafe { libc::sigaction(libc::SIGSEGV, &action as _, std::ptr::null_mut()) };
        assert_eq!(res, 0);
    }
}

fn setup_var_from_single_and_only_thread() {
    const VARS: &[(&str, &str)] = &[("RUST_BACKTRACE", "full")];

    for (name, value) in VARS {
        if std::env::var(name).is_err() {
            // Safe to call, we didn't launch any threads yet
            unsafe { std::env::set_var(name, value) };
        }
    }
}

/// Mimic default hook:
/// https://github.com/rust-lang/rust/blob/5986ff05d8480da038dd161b3a6aa79ff364a851/library/std/src/panicking.rs#L246
///
/// Unlike the default hook, this one allocates.
/// We store (+ display) panics in non-main threads, and display them all when the main thread panics.
#[cfg(not(target_family = "wasm"))]
fn new_hook(info: &PanicHookInfo<'_>) {
    use std::any::Any;
    use std::io::Write;

    fn payload_as_str(payload: &dyn Any) -> &str {
        if let Some(&s) = payload.downcast_ref::<&'static str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        }
    }

    static PREVIOUS_PANICS: std::sync::Mutex<Vec<Vec<u8>>> =
        const { std::sync::Mutex::new(Vec::new()) };

    let mut s: Vec<u8> = Vec::with_capacity(64 * 1024);
    let backtrace = Backtrace::new();

    let current = std::thread::current();
    let name = current.name().unwrap_or("<unnamed>");
    let location = info.location().unwrap();
    let msg = payload_as_str(info.payload());

    let _ = writeln!(&mut s, "\nthread '{name}' panicked at {location}:\n{msg}");
    let _ = writeln!(&mut s, "{:#?}", &backtrace);

    eprintln!("{}", String::from_utf8_lossy(&s));

    if name != "main" {
        let Ok(mut previous) = PREVIOUS_PANICS.lock() else {
            return;
        };
        // Make sure we don't store too many panics
        if previous.len() < 256 {
            previous.push(s);
            eprintln!("Saved panic from thread '{name}'");
        } else {
            eprintln!("Panic from thread '{name}' not saved !");
        }
    } else {
        let Ok(previous) = PREVIOUS_PANICS.lock() else {
            return;
        };
        eprintln!("\nNumber of panics from others threads: {}", previous.len());
        for panic in previous.iter() {
            eprintln!("{}", String::from_utf8_lossy(panic));
        }
    }
}

fn early_setup() {
    setup_var_from_single_and_only_thread();
    #[cfg(not(target_family = "wasm"))]
    std::panic::set_hook(Box::new(new_hook));
}

fn main() -> anyhow::Result<()> {
    early_setup();

    #[cfg(feature = "unsafe-signal-handlers")]
    unsafe_signal_handlers::setup();
    let app = commands::OpenminaCli::parse();

    let network_init_result = match app.network {
        commands::Network::Devnet => openmina_core::NetworkConfig::init("devnet"),
        commands::Network::Mainnet => openmina_core::NetworkConfig::init("mainnet"),
    };

    network_init_result.expect("Failed to initialize network configuration");

    app.command.run()
}
