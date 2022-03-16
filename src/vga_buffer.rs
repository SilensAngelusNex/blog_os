use core::{fmt, ops::DerefMut};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts::without_interrupts;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: LAST_ROW,
        col_position: 0,
        pallet: DEFAULT_COLORS,
        /// Safety:
        ///     This is a magical physical memory address, so there's always a Buffer there.
        ///     `lazy_static` ensures that only one mutable reference is ever created.
        buffer: unsafe { &mut *BUFFER },
    });
}

const BUFFER: *mut Buffer = 0xb8000 as _;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const LAST_ROW: usize = BUFFER_HEIGHT - 1;

const PLACEHOLDER_CHAR: u8 = 0xfe;
const DEFAULT_COLORS: Pallet = Pallet {
    text: ColorCode::new(Color::Green, Color::Black),
    error: ColorCode::new(Color::LightRed, Color::DarkGray),
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Describes the colors used in a VGA cell.
///
/// Bytes 0..3 contain the foreground color and byte 3 controls foreground intensity (dark/light).
/// Bytes 4..7 contain the background color, and byte 7 either controls background intensity or
/// determiners whether the foreground character should blink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

/// The contents of a single cell of the VGA buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

/// The VGA buffer
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[derive(Clone, Copy)]
/// The classes of text that can be written to the VGA Buffer.
pub enum TextType {
    Text,
    Error,
}

/// Describes the colors to use for different `TextType`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pallet {
    pub text: ColorCode,
    pub error: ColorCode,
}

/// Writes to the VGA Buffer.
pub struct Writer {
    row_position: usize,
    col_position: usize,
    pallet: Pallet,
    buffer: &'static mut Buffer,
}

/// Wraps `Writer` to provide a `fmt::Write` implementation that can be used for panic messages.
struct PanicWriter<'a> {
    writer: &'a mut Writer,
    old_location: (usize, usize),
}

impl ColorCode {
    /// Creates a `ColorCode` from a foreground and background `Color`.
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

impl Pallet {
    pub fn color(&self, ty: TextType) -> ColorCode {
        use TextType::*;
        match ty {
            Text => self.text,
            Error => self.error,
        }
    }
}

impl Writer {
    /// Writes a string `s` to the VGA buffer, using the color for `ty`.
    ///
    /// Any non-ASCII bytes in `s` are replaced with `PLACEHOLDER_CHAR`.
    pub fn write_string(&mut self, s: &str, ty: TextType) {
        self.write_bytes(s.bytes().map(byte_to_ascii), ty);
    }

    /// Writes a sequence of Code Page 437 characters `bytes` to the VGA Buffer, using the color for `ty`.
    pub fn write_bytes<I: IntoIterator<Item = u8>>(&mut self, bytes: I, ty: TextType) {
        for byte in bytes {
            self.write_byte(byte, ty);
        }
    }

    /// Writes a single byte to the `Writer`'s current location, then advances to the next location.
    fn write_byte(&mut self, byte: u8, ty: TextType) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.col_position;
                let color_code = self.pallet.color(ty);

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code,
                });
                self.col_position += 1;
            }
        }
    }

    /// Proceeds to the next row of the buffer. If the Writer is already on the last line, `shift_up` to make room.
    fn new_line(&mut self) {
        if self.row_position == LAST_ROW {
            self.shift_up();
        } else {
            self.row_position += 1;
        }

        self.col_position = 0;
    }

    /// Shifts all rows up by one, filling the bottom row with the background color for `Text`.
    fn shift_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(ScreenChar {
                ascii_char: b' ',
                color_code: self.pallet.color(TextType::Text),
            })
        }
    }
}

fn byte_to_ascii(unicode_byte: u8) -> u8 {
    match unicode_byte {
        // We can print printable ASCII (space through tilde) and newline
        b' '..=b'~' | b'\n' => unicode_byte,
        // For anything else, just print a placeholder
        _ => PLACEHOLDER_CHAR,
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s, TextType::Text);
        Ok(())
    }
}

impl<'a> PanicWriter<'a> {
    fn new(writer: &'a mut Writer) -> Self {
        let old_location = (writer.row_position, writer.col_position);
        writer.row_position = 0;
        writer.col_position = 0;

        Self {
            writer,
            old_location,
        }
    }
}

impl<'a> core::ops::Drop for PanicWriter<'a> {
    fn drop(&mut self) {
        let (old_row, old_col) = self.old_location;
        self.writer.row_position = old_row;
        self.writer.col_position = old_col;
    }
}

impl<'a> fmt::Write for PanicWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_string(s, TextType::Error);
        Ok(())
    }
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write as _;

    without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! panic_print {
    ($($arg:tt)*) => ($crate::vga_buffer::_panic_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _panic_print(args: fmt::Arguments) {
    use core::fmt::Write as _;

    without_interrupts(|| {
        PanicWriter::new(WRITER.lock().deref_mut())
            .write_fmt(args)
            .unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    const MAX: usize = 200;
    for i in 1..=MAX {
        println!("Printing many lines: {} of {}", i, MAX);
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write as _;

    let s = "Some test string that fits on one line.";
    more_asserts::assert_le!(s.len(), BUFFER_WIDTH);
    assert!(s.is_ascii());

    without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (i, c) in s.chars().enumerate() {
            let byte = writer.buffer.chars[LAST_ROW - 1][i].read().ascii_char;
            assert_eq!(char::from(byte), c);
        }
    });
}
