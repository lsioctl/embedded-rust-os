use x86_64::instructions::port::{PortGeneric, ReadOnlyAccess};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;

use pic8259::ChainedPics;
use spin;

// import from this lib crate
use crate::{println, print};
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

extern "x86-interrupt" fn timer_interrupt_handler(_: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_: InterruptStackFrame) {
    // we need to read the keycode
    use x86_64::instructions::port::Port;

    // the data port is PS/2 IO port at address 0x60
    // TODO: Why crate x86_64 defaults to PortGeneric<_, ReadWriteAccess> as it
    // should be a ReadOnly ?
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe {
        port.read()
    };

    // by default PS/2 emulates scancode set 1 (IBM XT)
    // https://wiki.osdev.org/Keyboard#Scan_Code_Set_1
    // lower 7 bits for the key
    // and most significant bit defining if is
    // a press (0) or a release (1)
    // keys that were not present on the original XT keyboard
    // like enter on the keypad generate two scancode
    
    let key = match scancode {
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0a => Some('9'),
        0x0b => Some('0'),
        _ => None,
    };

    if let Some(character) = key {
        println!{"{}", character};
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
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

        //idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

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

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// the Mutex will allow us safe mutable access
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    // wrong offset would cause UB so unsafe
    unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    }
);

#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    // no need for value it will be Timer + 1
    Keyboard
}

impl InterruptIndex {
    pub fn as_usize(self) -> usize {
        usize::from(self as u8)
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}