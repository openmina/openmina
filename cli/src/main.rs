pub mod commands;
use clap::Parser;
pub use commands::CommandError;

mod exit_with_error;
pub use exit_with_error::exit_with_error;

use nix::libc;

extern "C" fn handle_sigsegv(_signal: libc::c_int) {
    eprintln!("########### handler #############");
    snarker::recorder::Recorder::crash_handler();
    std::process::exit(1);
}

fn setup_sigsegv_handler() {
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

fn main() {
    setup_sigsegv_handler();
    // unsafe {
    //     use signal_hook::consts::SIGSEGV;
    //     signal_hook_registry::register_unchecked(SIGSEGV, |_| {
    //         eprintln!("crash handlerr!!!!");
    //         snarker::recorder::Recorder::crash_handler();
    //     }).unwrap();
    // }

    match commands::OpenminaCli::parse().command.run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}
