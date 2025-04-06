use x86_64::{
    instructions::{hlt, interrupts, tlb},
    registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags, Efer, EferFlags},
    structures::paging::PhysFrame,
    PhysAddr
};

use crate::ram::{PAGE_2MIB, PAGE_4KIB};

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
const ENTRIES_PER_TABLE: u64 = 512;

pub unsafe fn map_identity(ram_size: u64) {
    let pml4_addr = PAGE_TABLE_ADDR;
    let pdpt_addr = pml4_addr + PAGE_4KIB as u64;
    let pd_addr = pdpt_addr + PAGE_4KIB as u64;

    for i in 0..3 {
        let table_addr = PAGE_TABLE_ADDR + (i * PAGE_4KIB as u64);
        for j in 0..ENTRIES_PER_TABLE {
            let entry_addr = table_addr + (j * 8);
            *(entry_addr as *mut u64) = 0;
        }
    }
    serial_print("Page tables initialized to zero\n");

    *(pml4_addr as *mut u64) = pdpt_addr | 0x3; // PRESENT | WRITABLE
    *(pdpt_addr as *mut u64) = pd_addr | 0x3;   // PRESENT | WRITABLE
    serial_print("Basic page structure set up\n");

    for i in 0..ram_size / PAGE_2MIB as u64 {
        let pd_entry_addr = pd_addr + (i * 8);
        let phys_addr = i * PAGE_2MIB as u64;
        *(pd_entry_addr as *mut u64) = phys_addr | 0x83; // PRESENT | WRITABLE | PAGE_2MIB
    }

    serial_print("2MB pages mapped for first 1GB\n");

    let mut cr4 = Cr4::read();
    cr4 |= Cr4Flags::PHYSICAL_ADDRESS_EXTENSION;
    cr4 |= Cr4Flags::PAGE_SIZE_EXTENSION;
    Cr4::write(cr4);

    let mut efer = Efer::read();
    efer |= EferFlags::LONG_MODE_ENABLE;
    efer |= EferFlags::NO_EXECUTE_ENABLE;
    Efer::write(efer);

    serial_print("CR4 and EFER set\n");

    serial_print("Setting CR3 to ");
    print_hex64(pml4_addr);
    serial_print("\n");

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
        0x180000,   // 1 MB
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
        }
        else {
            serial_print("FAILED (read 0x");
            print_hex64(read_value as u64);
            serial_print(")\n");
        }
    }
}