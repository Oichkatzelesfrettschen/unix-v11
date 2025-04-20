use x86_64::{
    instructions::{hlt, interrupts, tlb},
    registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags, Efer, EferFlags},
    structures::paging::PhysFrame,
    PhysAddr
};

use crate::ram::PAGE_4KIB;

pub fn halt() {
    interrupts::disable();
    hlt();
}

const COM1: u16 = 0x3F8;

pub fn init_serial() {
    unsafe {
        use x86_64::instructions::port::Port;

        let mut port = Port::new(COM1 + 1);
        port.write(0x00u8); // Disable all interrupts
        let mut port = Port::new(COM1 + 3);
        port.write(0x80u8); // Enable DLAB (set baud rate divisor)
        let mut port = Port::new(COM1 + 0);
        port.write(0x03u8); // Set divisor to 3 (lo byte) 38400 baud
        let mut port = Port::new(COM1 + 1);
        port.write(0x00u8); //                  (hi byte)
        let mut port = Port::new(COM1 + 3);
        port.write(0x03u8); // 8 bits, no parity, one stop bit
        let mut port = Port::new(COM1 + 2);
        port.write(0xC7u8); // Enable FIFO, clear them, with 14-byte threshold
        let mut port = Port::new(COM1 + 4);
        port.write(0x0Bu8); // IRQs enabled, RTS/DSR set
    }
}

pub fn serial_write_byte(byte: u8) {
    unsafe {
        use x86_64::instructions::port::Port;

        let mut line_status = Port::<u8>::new(COM1 + 5);
        while line_status.read() & 0x20 == 0 {}

        let mut data = Port::<u8>::new(COM1);
        data.write(byte);
    }
}

pub fn serial_print(s: &str) {
    for b in s.bytes() {
        serial_write_byte(b);
    }
}

pub fn print_hex64(num: u64) {
    (0..16).rev().for_each(|i| {
        let nibble = (num >> (i * 4)) & 0xF;
        let hex_char = b"0123456789abcdef"[nibble as usize];
        serial_write_byte(hex_char);
    });
}

const PAGE_TABLE_ADDR: u64 = 0x200000;
const ENTRIES_PER_TABLE: usize = 0x200;

pub unsafe fn map_identity(ram_size: u64) {
    let pml4_addr = PAGE_TABLE_ADDR;

    // Calculate counts
    let num_4kib_pages = (ram_size as usize + PAGE_4KIB - 1) / PAGE_4KIB;
    let num_pt = (num_4kib_pages + ENTRIES_PER_TABLE - 1) / ENTRIES_PER_TABLE;
    let num_pd = (num_pt + ENTRIES_PER_TABLE - 1) / ENTRIES_PER_TABLE;
    let num_pdpt = (num_pd + ENTRIES_PER_TABLE - 1) / ENTRIES_PER_TABLE;

    let table_size = (1 + num_pdpt + num_pd + num_pt) * PAGE_4KIB;

    let pdpt_base = pml4_addr + PAGE_4KIB as u64;
    let pd_base = pdpt_base + (num_pdpt as u64 * PAGE_4KIB as u64);
    let pt_base = pd_base + (num_pd as u64 * PAGE_4KIB as u64);

    for i in 0..ENTRIES_PER_TABLE { // Zero out PML4
        let entry = (pml4_addr + (i as u64 * 8)) as *mut u64;
        *entry = 0;
    }

    for t in 0..num_pdpt { // Zero out PDPTs
        let table_addr = pdpt_base + (t as u64 * PAGE_4KIB as u64);
        for i in 0..ENTRIES_PER_TABLE {
            let entry = (table_addr + (i as u64 * 8)) as *mut u64;
            *entry = 0;
        }
    }

    for t in 0..num_pd { // Zero out PDs
        let table_addr = pd_base + (t as u64 * PAGE_4KIB as u64);
        for i in 0..ENTRIES_PER_TABLE {
            let entry = (table_addr + (i as u64 * 8)) as *mut u64;
            *entry = 0;
        }
    }

    for i in 0..num_pdpt { // Link PML4 -> PDPTs
        let pml4_entry = (pml4_addr + (i as u64 * 8)) as *mut u64;
        *pml4_entry = (pdpt_base + (i as u64 * PAGE_4KIB as u64)) | 0x3;
    }

    for i in 0..num_pdpt { // Link PDPTs -> PDs
        let pdpt_entry_addr = pdpt_base + (i as u64 * PAGE_4KIB as u64);
        for j in 0..ENTRIES_PER_TABLE {
            let pdpt_entry = (pdpt_entry_addr + (j as u64 * 8)) as *mut u64;
            let pd_index = i * ENTRIES_PER_TABLE + j;
            if pd_index >= num_pd { break; }
            *pdpt_entry = (pd_base + (pd_index as u64 * PAGE_4KIB as u64)) | 0x3;
        }
    }

    for i in 0..num_pd { // Link PDs -> PTs
        let pd_entry_addr = pd_base + (i as u64 * PAGE_4KIB as u64);
        for j in 0..ENTRIES_PER_TABLE {
            let pd_entry = (pd_entry_addr + (j as u64 * 8)) as *mut u64;
            let pt_index = i * ENTRIES_PER_TABLE + j;
            if pt_index >= num_pt { break; }
            *pd_entry = (pt_base + (pt_index as u64 * PAGE_4KIB as u64)) | 0x3;
        }
    }

    let mut phys = 0u64;
    for pt_idx in 0..num_pt { // Allocate Pages
        let pt_table_addr = pt_base + (pt_idx as u64 * PAGE_4KIB as u64);
        for j in 0..ENTRIES_PER_TABLE {
            if phys >= ram_size { break; }
            let entry = (pt_table_addr + (j as u64 * 8)) as *mut u64;
            *entry = phys | 0x03; // PRESENT | WRITABLE
            phys += PAGE_4KIB as u64;
        }
    }

    let mut cr4 = Cr4::read();
    cr4 |= Cr4Flags::PHYSICAL_ADDRESS_EXTENSION | Cr4Flags::PAGE_SIZE_EXTENSION;
    Cr4::write(cr4);

    let mut efer = Efer::read();
    efer |= EferFlags::LONG_MODE_ENABLE | EferFlags::NO_EXECUTE_ENABLE;
    Efer::write(efer);

    serial_print("Table size = 0x");
    print_hex64(table_size as u64);
    serial_print(" bytes\n");

    Cr3::write(
        PhysFrame::containing_address(PhysAddr::new(pml4_addr)),
        Cr3Flags::empty()
    );

    tlb::flush_all();
    serial_print("TLB flushed\n");
    test_memory_areas();
}

unsafe fn test_memory_areas() {
    let test_addresses = [
        0x300000,   // 3 MB
        0x500000,   // 5 MB
        0xa00000,   // 10 MB
        0x1000000,  // 16 MB
        0x5000000,  // 80 MB
    ];

    for &addr in &test_addresses {
        serial_print("Testing memory at 0x");
        print_hex64(addr);
        serial_print("... ");

        let test_value = 0xFEEDFACECAFEBABEu64;
        let test_ptr = addr as *mut u64;

        core::ptr::write_volatile(test_ptr, test_value);
        let read_value = core::ptr::read_volatile(test_ptr);

        if read_value == test_value {
            serial_print("SUCCESS\n");
        } else {
            serial_print("FAILED (read 0x");
            print_hex64(read_value as u64);
            serial_print(")\n");
        }
    }
}