//!                         Research UNIX Version 11                         !//
//!
//! Made by HaמuL in 2025
//! Description: Inchoate entry point of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]
extern crate alloc;

mod ram;

use core::panic::PanicInfo;
use uefi::{entry, println, Status};
macro_rules! arch {
    ($arch:literal, $modname:ident) => {
        #[cfg(target_arch = $arch)]
        mod $modname;
        #[cfg(target_arch = $arch)]
        use $modname as arch;
    };
}

arch!("aarch64", aarch64);
arch!("x86_64", amd64);


fn init_storage() {
    // init_devices();
    // init_filesys();
}

fn init_metal() {
    ram::init_ram();
    init_storage();
}
fn exec_aleph() {}
fn schedule() -> ! { loop { arch::halt(); } }

#[entry]
fn ignite() -> Status {
    init_metal();
    // load_kernel_image();
    // exit_boot_services();
    // jump_to_kernel();

    // init_metal();
    exec_aleph();
    schedule();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop { arch::halt(); }
}