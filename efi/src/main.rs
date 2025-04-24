//!                              EFI Bootloader                              !//
//!
//! Crafted by HaמuL in 2025
//! Description: EFI Bootloader of Research UNIX Version 11
//! Licence: Public Domain

#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::{copy_nonoverlapping, write_bytes}, slice::from_raw_parts_mut};
use uefi::{
    boot::{allocate_pages, exit_boot_services, get_image_file_system, image_handle, memory_map, AllocateType, MemoryType},
    cstr16, entry, mem::memory_map::MemoryMap, println, proto::media::file::{File, FileAttribute, FileInfo, FileMode}, Status
};
use xmas_elf::{program::Type, ElfFile};

const PAGE_4KIB: usize = 0x1000;

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
    let efi_ram_layout = memory_map(MemoryType::LOADER_DATA).unwrap();
    let descriptor_largest = efi_ram_layout.entries()
        .filter(|e| e.ty == MemoryType::CONVENTIONAL)
        .max_by_key(|e| e.page_count).unwrap();
    let ram_base = descriptor_largest.phys_start;

    let mut filesys_protocol = get_image_file_system(image_handle()).unwrap();
    let mut root = filesys_protocol.open_volume().unwrap();

    let mut file = root.open(
        cstr16!("\\bin\\unix-v11"), FileMode::Read, FileAttribute::empty()
    ).unwrap().into_regular_file().unwrap();

    let mut info_buf = [0u8; 512];
    let info = file.get_info::<FileInfo>(&mut info_buf).unwrap();
    let size = info.file_size() as usize;

    let pages = (size + PAGE_4KIB - 1) / PAGE_4KIB;
    let ptr = allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages).unwrap();
    let kernel = unsafe { from_raw_parts_mut(ptr.as_ptr(), size) };
    file.read(kernel).unwrap();

    let elf = ElfFile::new(&kernel).unwrap();
    for ph in elf.program_iter() {
        if let Ok(Type::Load) = ph.get_type() {
            let mem_size = ph.mem_size() as usize;
            let phys_addr = (ram_base + ph.physical_addr()) as *mut u8;
            let offset = ph.offset() as usize;
            let file_size = ph.file_size() as usize;

            unsafe {
                copy_nonoverlapping(kernel[offset..].as_ptr(), phys_addr, file_size);
                write_bytes(phys_addr.add(file_size), 0, mem_size - file_size);
            }
        }
    }

    let entrypoint = elf.header.pt2.entry_point() as usize + ram_base as usize;
    let jump: extern "C" fn(*const RAMDescriptor, usize) -> ! = unsafe { core::mem::transmute(entrypoint) };
    let efi_ram_layout = unsafe { exit_boot_services(MemoryType::LOADER_DATA) };
    jump(efi_ram_layout.buffer().as_ptr() as *const RAMDescriptor, efi_ram_layout.len());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop { arch::halt(); }
}