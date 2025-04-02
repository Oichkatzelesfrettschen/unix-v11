use uefi::boot::{MemoryDescriptor, MemoryType};
use linked_list_allocator::LockedHeap;

pub const HEAP_SIZE: usize = 0x10_0000;
const PAGE_SIZE: usize = 0x1000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_ram(efi_ram_layout: &[MemoryDescriptor]) {
    let heap_pages = HEAP_SIZE / PAGE_SIZE;
    let mut heap_start: Option<usize> = None;

    for entry in efi_ram_layout.iter() {
        if entry.ty != MemoryType::CONVENTIONAL { continue; }

        if entry.page_count as usize >= heap_pages {
            let region_start = entry.phys_start as usize;
            heap_start = Some(region_start);
            break;
        }
    }

    let heap_start = heap_start.expect("Not enough memory for heap");
    unsafe { ALLOCATOR.lock().init(heap_start as *mut u8, HEAP_SIZE); }
}