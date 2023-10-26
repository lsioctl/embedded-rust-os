use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;

// import from this lib crate
use crate::println;
use crate::gdt;

// FFI
// Foreign calling convention
// for interrupt calling convention (different prologue ...)
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception breakpoint, stack frame: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault triggered, stack frame: {:#?}", stack_frame);
}

// we ne once more lazy_static to avoid mutable static
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        let double_fault_entry = idt.double_fault.set_handler_fn(double_fault_handler);
        // unsafe because we have to ensure the index is valid
        // and not already used for another exception
        unsafe {
            double_fault_entry.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt
    };
}

pub fn init_idt() {
    // load the IDT in the cpu with lidt
    // loads needs a &'static self
    // as the cpu will access this table
    // for every interrupts
    // until another IDT is loaded
    IDT.load();  
}

