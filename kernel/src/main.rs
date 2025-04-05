//!          Uniplexed Information and Computing Service Version 11          !//
//!
//! Crafted by HaמuL in 2025
//! Description: Kernel of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]
extern crate alloc;

mod ram;

use core::panic::PanicInfo;
use uefi::boot::MemoryDescriptor;

macro_rules! arch {
    ($arch:literal, $modname:ident) => {
        #[cfg(target_arch = $arch)] mod $modname;
        #[cfg(target_arch = $arch)] use $modname as arch;
    };
}

arch!("aarch64", aarch64);
arch!("x86_64", amd64);

fn init_metal(efi_ram_layout: &[MemoryDescriptor]) {
    ram::init_ram(efi_ram_layout);
    // init_storage();
}
fn exec_aleph() {}
fn schedule() -> ! { loop { arch::halt(); } }

#[repr(C)]
pub struct KernelArgs {
    pub layout_ptr: *const MemoryDescriptor,
    pub layout_len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn ignite(args: KernelArgs) -> ! {
    arch::init_serial();
    arch::serial_print("Research UNIX Version 11\n");
    let efi_ram_layout = unsafe { core::slice::from_raw_parts(args.layout_ptr, args.layout_len) };
    init_metal(efi_ram_layout);
    arch::serial_print("EFI RAM Init success\n");
    exec_aleph();
    schedule();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { arch::halt(); }
}