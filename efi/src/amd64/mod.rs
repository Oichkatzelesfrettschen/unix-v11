use x86_64::instructions::{hlt, interrupts};

pub fn halt() {
    interrupts::disable();
    hlt();
}