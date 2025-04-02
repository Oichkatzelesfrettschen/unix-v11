pub fn halt() {
    aarch64::irq::nested_disable();
    aarch64::instructions::halt();
}