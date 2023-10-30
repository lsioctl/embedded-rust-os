use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// u4 would have been sufficient, but doesn't exist
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// ensure that the type has the same data layout
// as the underlying type (u8)
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // recreate the VGA buffer color which contains first bacground color on 4 bits
        // and then the font color on 4 bits
        // so shift left and or does the job
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Since Rust does not guarantees field ordering, switch to C layout
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// ensure it has the same memory layout
// than a single of its fields
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            _ => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;

                // note sur this is done the right way
                // but the blog recommended to use volatile <= 2.6
                // and for me it is no go to use such an outdated crate
                let dst = &mut self.buffer.chars[row][col];
                let src = ScreenChar {
                    ascii_character: byte,
                    color_code
                };
                // for me it is the minimum unsafe portion possible with write_volatie
                // * dst is still bound checked (outside of unsafe)
                // * src is outside of unsafe (not sure what it brings as a safety)
                unsafe {
                    core::ptr::write_volatile(
                        // &mut T automatically converts to *mut T
                        dst,
                        src
                    );
                };

                self.column_position += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        // TODO: real scrolling so shift the matrix rows
        if self.row_position >= BUFFER_HEIGHT - 1 {
            (1..BUFFER_HEIGHT).for_each(|row| {
                (0..BUFFER_WIDTH).for_each(|col| {
                    // I can't do something like this:
                    // let src = & self.buffer.chars[row][col];
                    // let dst = &mut self.buffer.chars[row - 1][col];
                    // unsafe {
                    //    let character = core::ptr::read_volatile(src);
                    //    core::ptr::write_volatile(dst, character);
                    // }
                    // cannot borrow `**self.buffer.chars[_][_]` as mutable because it is also borrowed as immutable
                    let src = & self.buffer.chars[row][col];
                    let character = unsafe { 
                        core::ptr::read_volatile(src)
                    };

                    let dst = &mut self.buffer.chars[row - 1][col];
                    unsafe {
                        core::ptr::write_volatile(dst, character);
                    }
                })
            });
            self.row_position = BUFFER_HEIGHT - 1;
            self.column_position = 0;
            self.write_byte(b'z');
        } else {
            self.row_position += 1;
            self.column_position = 0;
            self.write_byte(b'u');
        }

        
    }

    pub fn write_string(&mut self, s: &str) {
        s.bytes().for_each(|byte| {
            match byte {
                // ASCII character range or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // print a square (last character of the code page 737
                // https://en.wikipedia.org/wiki/Code_page_737
                // If we have an utf8 character, (so multi byte, and could be
                // of various length), individual bytes of the char are guaranteed
                // to no be valid ASCII
                // so each byte would land on this match arm
                _ => self.write_byte(0xfe)
            }
        });
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// we use lazy_static to have a safe mutable global variable
// we can't have it with const fn because we create a mutable
// reference from a row pointer, and that can't be done
// at compile time. Note that the lazy_static we use do
// not work with the heap (we have no heap for now)
// so it uses spin crate's Once
lazy_static! {
    // we use Mutex from spin crate (spinlock)
    // has we don't use std, and have no thread
    pub static ref WRITER: Mutex<Writer> = Mutex::new(
        Writer {
            row_position: 0,
            column_position: 0,
            color_code: ColorCode::new(Color::LightRed, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    );
}


// macros inspired from std println! macro
// implementation so hidden from the doc
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

