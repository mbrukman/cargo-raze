extern crate rayon_core;

use rayon_core::ThreadPoolBuilder;

use std::process::Command;
use std::env;

#[cfg(target_os = "linux")]
use std::os::unix::process::ExitStatusExt;

fn force_stack_overflow(depth: u32) {
    let _buffer = [0u8; 1024 * 1024];
    if depth > 0 {
        force_stack_overflow(depth - 1);
    }
}

fn main() {
    if env::args().len() == 1 {
        // first check that the recursivecall actually causes a stack overflow, and does not get optimized away
        {
            let status = Command::new(env::current_exe().unwrap())
                .arg("8")
                .status()
                .unwrap();

            #[cfg(windows)]
            assert_eq!(status.code(), Some(0xc00000fd /*STATUS_STACK_OVERFLOW*/));

            #[cfg(unix)]
            assert_eq!(status.code(), None);

            #[cfg(target_os = "linux")]
            assert!(
                status.signal() == Some(11 /*SIGABRT*/) || status.signal() == Some(6 /*SIGSEGV*/)
            );
        }

        // now run with a larger stack and verify correct operation
        {
            let status = Command::new(env::current_exe().unwrap())
                .arg("48")
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(0));
            #[cfg(target_os = "linux")]
            assert_eq!(status.signal(), None);
        }
    } else {
        let stack_size_in_mb: usize = env::args().nth(1).unwrap().parse().unwrap();
        let pool = ThreadPoolBuilder::new()
            .stack_size(stack_size_in_mb * 1024 * 1024)
            .build()
            .unwrap();
        pool.install(|| {
            force_stack_overflow(32);
        });
    }
}
