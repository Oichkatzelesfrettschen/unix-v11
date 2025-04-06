//!                              EFI Bootloader                              !//
//!
//! Crafted by HaמuL in 2025
//! Description: EFI Bootloader of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]

mod storage;

use core::panic::PanicInfo;
use uefi::{
    boot::{exit_boot_services, MemoryType},
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
#[derive(Clone, Copy, Debug)]
pub struct RAMDescriptor {
    pub ty: u32,
    pub reserved: u32,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_count: u64,
    pub attr: u64,
    pub padding: u64
}

#[entry]
fn ignite() -> Status {
    let ptr = storage::load_kernel_image();
    let efi_ram_layout = unsafe { exit_boot_services(MemoryType::LOADER_DATA) };
    let jump: extern "C" fn(*const RAMDescriptor, usize) -> ! = unsafe { core::mem::transmute(ptr) };
    jump(efi_ram_layout.buffer().as_ptr() as *const RAMDescriptor, efi_ram_layout.len());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop { arch::halt(); }
}