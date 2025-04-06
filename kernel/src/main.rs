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
use ram::{RAMDescriptor, PAGE_4KIB};

macro_rules! arch {
    ($arch:literal, $modname:ident) => {
        #[cfg(target_arch = $arch)] mod $modname;
        #[cfg(target_arch = $arch)] use $modname as arch;
    };
}

arch!("aarch64", aarch64);
arch!("x86_64", amd64);

fn init_metal(efi_ram_layout: &[RAMDescriptor]) {
    let last_ram_desc = efi_ram_layout[efi_ram_layout.len() - 1];
    let ram_size = last_ram_desc.phys_start + last_ram_desc.page_count * PAGE_4KIB as u64;
    arch::serial_print("RAM Size: ");
    arch::print_hex64(ram_size);
    arch::serial_print("\n");
    unsafe { arch::map_identity(ram_size); }
    ram::init_ram(efi_ram_layout);
    // init_storage();
}
fn exec_aleph() {}
fn schedule() -> ! { loop { arch::halt(); } }

#[no_mangle]
pub extern "win64" fn ignite(layout_ptr: *const RAMDescriptor, layout_len: usize) -> ! {
    arch::init_serial();
    arch::serial_print("Research UNIX Version 11\n");
    let efi_ram_layout = unsafe { core::slice::from_raw_parts(layout_ptr, layout_len) };
    init_metal(efi_ram_layout);
    exec_aleph();
    schedule();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { arch::halt(); }
}