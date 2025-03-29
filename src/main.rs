//!                         Research UNIX Version 11                         !//
//!
//! Made by HaמuL in 2025
//! Description: Inchoate entry point of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]
extern crate alloc;

mod archs; mod ram;

use core::panic::PanicInfo;
use uefi::{entry, println, Status};
#[cfg(target_arch = "aarch64")]
use archs::aarch64 as arch;
#[cfg(target_arch = "x86_64")]
use archs::amd64 as arch;

fn init_storage() {}

fn init_metal() {
    ram::init_ram();
    init_storage();
}
fn exec_aleph() {}
fn schedule() -> ! { loop { arch::halt(); } }

#[entry]
fn ignite() -> Status {
    init_metal();
    exec_aleph();
    schedule();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop { arch::halt(); }
}