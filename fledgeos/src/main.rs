#![no_std]
#![no_main]
#![feature(core_intrinsics)]
// Rust一些元素是通过编译器之外的库的形式实现的，我们禁用了标准库，所以要自己实现一些这部分的功能
#![feature(lang_items)]

use core::fmt::Write;
use core::intrinsics;
use core::panic::PanicInfo;
use x86_64::instructions::hlt;

#[allow(unused)]
#[derive(Copy, Clone)]
#[repr(u8)] // 用一个字节来表示这些值
enum Color {
    Black = 0x0,    White = 0xF,
    Blue = 0x1,     BrightBlue = 0x9,
    Green = 0x2,    BrightGreen = 0xA,
    Cyan = 0x3,     BrightCyan = 0xB,
    Red = 0x4,      BrightRed = 0xC,
    Magenta = 0x5,  BrightMagenta = 0xD,
    Brown = 0x6,    Yellow = 0xE,
    Gray = 0x7,     DarkGray = 0x8
}

struct Cursor {
    position: isize,
    foreground: Color,
    background: Color,
}

impl Cursor {
    fn color(&self) -> u8 {
        let fg = self.foreground as u8;
        let bg = (self.background as u8) << 4;
        fg | bg
    }

    fn print(&mut self, text: &[u8]) {
        let color = self.color();

        let framebuffer = 0xb8000 as *mut u8;

        for &character in text {
            unsafe {
                framebuffer.offset(self.position).write_volatile(character);
                framebuffer.offset(self.position + 1).write_volatile(color);
            }
            self.position += 2;
        }
    }
}

impl Write for Cursor {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s.as_bytes());
        Ok(())
    }
}

// 这是一个lang item，异常时会在每个栈帧上调用这个函数来进行stack unwinding，eh = expect handler
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    let mut cursor = Cursor {
        position: 0,
        foreground: Color::White,
        background: Color::Red,
    };
    for _ in 0..(80*25) {
        cursor.print(b" ");
    }
    cursor.position = 0;
    write!(cursor, "{}", info).unwrap();

    loop {
        hlt();
    }
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut cursor = Cursor {
        position: 0,
        foreground: Color::BrightCyan,
        background: Color::Black,
    };
    cursor.print(HELLO);

    // panic!("help");
    loop {
        hlt();
    }
}