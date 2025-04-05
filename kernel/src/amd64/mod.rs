use x86_64::instructions::{hlt, interrupts};

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