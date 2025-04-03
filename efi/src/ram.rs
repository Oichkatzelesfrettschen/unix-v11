use uefi::boot::{allocate_pages, AllocateType, MemoryType};
use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = 0x10_0000;
const PAGE_SIZE: usize = 0x1000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_ram() {
    let pages_count = (HEAP_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
    let ptr = allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages_count
    ).unwrap();

    unsafe { ALLOCATOR.lock().init(ptr.as_ptr(), HEAP_SIZE); }
}