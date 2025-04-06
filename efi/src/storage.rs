use core::{ptr::{copy_nonoverlapping, write_bytes}, slice::from_raw_parts_mut};
use uefi::{
    boot::{allocate_pages, get_image_file_system, image_handle, AllocateType, MemoryType},
    cstr16, proto::media::file::{File, FileAttribute, FileInfo, FileMode}
};
use xmas_elf::{program::Type, ElfFile};

const PAGE_4KIB: usize = 0x1000;

pub fn load_kernel_image() -> usize {
    let mut filesys_protocol = get_image_file_system(image_handle()).unwrap();
    let mut root = filesys_protocol.open_volume().unwrap();

    let mut file = root.open(
        cstr16!("bin\\unix-v11"),
        FileMode::Read,
        FileAttribute::empty()
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
            let phys_addr = ph.physical_addr() as *mut u8;
            let offset = ph.offset() as usize;
            let file_size = ph.file_size() as usize;

            unsafe {
                copy_nonoverlapping(kernel[offset..].as_ptr(), phys_addr, file_size);
                write_bytes(phys_addr.add(file_size), 0, mem_size - file_size);
            }
        }
    }

    return elf.header.pt2.entry_point() as usize;
}