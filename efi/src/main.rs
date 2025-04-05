//!                 Research UNIX Version 11 EFI Application                 !//
//!
//! Made by HaמuL in 2025
//! Description: EFI Bootloader of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]

mod storage;

use core::panic::PanicInfo;
use uefi::{
    boot::{exit_boot_services, MemoryDescriptor, MemoryType},
    entry, mem::memory_map::MemoryMap, println, Status
};

macro_rules! arch {
    ($arch:literal, $modname:ident) => {
        #[cfg(target_arch = $arch)] mod $modname;
        #[cfg(target_arch = $arch)] use $modname as arch;
    };
}

arch!("aarch64", aarch64);
arch!("x86_64", amd64);

#[repr(C)]
pub struct KernelArgs {
    pub layout_ptr: *const MemoryDescriptor,
    pub layout_len: usize,
}

#[entry]
fn ignite() -> Status {
    let ptr = storage::load_kernel_image();
    let efi_ram_layout = unsafe { exit_boot_services(MemoryType::LOADER_DATA) };
    let arg = KernelArgs {
        layout_ptr: efi_ram_layout.buffer().as_ptr() as *const MemoryDescriptor,
        layout_len: efi_ram_layout.len(),
    };
    let jump: extern "C" fn(arg: KernelArgs) -> ! = unsafe { core::mem::transmute(ptr) };
    jump(arg);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop { arch::halt(); }
}