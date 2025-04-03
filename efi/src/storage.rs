use alloc::vec::Vec;
use uefi::{boot, cstr16, proto::media::file::{File, FileAttribute, FileInfo, FileMode}};
use xmas_elf::program::Type;

pub fn load_kernel_image() -> usize {
    let mut filesys_protocol = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut root = filesys_protocol.open_volume().unwrap();

    let mut file = root.open(
        cstr16!("bin\\unix-v11"),
        FileMode::Read,
        FileAttribute::empty()
    ).unwrap().into_regular_file().unwrap();

    let mut info_buf = [0u8; 512];
    let info = file.get_info::<FileInfo>(&mut info_buf).unwrap();
    let size = info.file_size() as usize;
    let mut buf = Vec::with_capacity(size);
    unsafe { buf.set_len(size); }

    let _ = file.read(&mut buf).unwrap();

    let elf = xmas_elf::ElfFile::new(&buf).unwrap();

    for ph in elf.program_iter() {
        if let Ok(Type::Load) = ph.get_type() {
            let virt_addr = ph.virtual_addr() as *mut u8;
            let offset = ph.offset() as usize;
            let file_size = ph.file_size() as usize;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    buf[offset..].as_ptr(),
                    virt_addr,
                    file_size,
                );
            }
        }
    }

    return elf.header.pt2.entry_point() as usize;
}