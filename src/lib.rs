#![no_std]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
mod interrupts;
mod gdt;

pub fn init() {
    interrupts::init_idt();
    gdt::init();
    // unsafe because init of wrongly configured PIC would be UB
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    // execute the special sti (set interrupts) instruction
    x86_64::instructions::interrupts::enable();
}
