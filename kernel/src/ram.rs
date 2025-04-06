use linked_list_allocator::LockedHeap;

// pub const HEAP_SIZE: usize = 0x10_0000;

pub const PAGE_4KIB: usize = 0x1000;
pub const PAGE_2MIB: usize = 0x200000;
// pub const PAGE_1GIB: usize = 0x40000000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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

pub fn init_ram(_efi_ram_layout: &[RAMDescriptor]) {}