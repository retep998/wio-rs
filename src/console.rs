// Copyright © 2016, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {Result, k32, last_error, w};
use handle::{Handle};
use std::mem::{size_of_val, zeroed};
use std::os::windows::io::{FromRawHandle};
use std::ptr::{null, null_mut};
use wide::{ToWide};

pub struct ScreenBuffer(Handle);
impl ScreenBuffer {
    pub fn new() -> Result<ScreenBuffer> {
        let handle = unsafe { k32::CreateConsoleScreenBuffer(
            w::GENERIC_READ | w::GENERIC_WRITE, w::FILE_SHARE_READ | w::FILE_SHARE_WRITE,
            null(), w::CONSOLE_TEXTMODE_BUFFER, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { return last_error() }
        unsafe { Ok(ScreenBuffer(Handle::new(handle))) }
    }
    /// Gets the actual active console screen buffer
    pub fn from_conout() -> Result<ScreenBuffer> {
        let handle = unsafe { k32::CreateFileW(
            "CONOUT$".to_wide_null().as_ptr(), w::GENERIC_READ | w::GENERIC_WRITE,
            w::FILE_SHARE_READ, null_mut(), w::OPEN_EXISTING,
            0, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { return last_error() }
        unsafe { Ok(ScreenBuffer(Handle::new(handle))) }
    }
    pub fn set_active(&self) -> Result<()> {
        let res = unsafe { k32::SetConsoleActiveScreenBuffer(*self.0) };
        if res == 0 { return last_error() }
        Ok(())
    }
    pub fn info(&self) -> Result<ScreenBufferInfo> {
        let mut info = ScreenBufferInfo(unsafe { zeroed() });
        let res = unsafe { k32::GetConsoleScreenBufferInfo(*self.0, &mut info.0) };
        if res == 0 { return last_error() }
        Ok(info)
    }
    pub fn info_ex(&self) -> Result<ScreenBufferInfoEx> {
        let mut info: w::CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { zeroed() };
        info.cbSize = size_of_val(&info) as u32;
        let res = unsafe { k32::GetConsoleScreenBufferInfoEx(*self.0, &mut info) };
        if res == 0 { return last_error() }
        // Yes, this is important
        info.srWindow.Right += 1;
        info.srWindow.Bottom += 1;
        Ok(ScreenBufferInfoEx(info))
    }
    pub fn set_info_ex(&self, mut info: ScreenBufferInfoEx) -> Result<()> {
        let res = unsafe { k32::SetConsoleScreenBufferInfoEx(*self.0, &mut info.0) };
        if res == 0 { return last_error() }
        Ok(())
    }
    // pub fn font_ex(&self) -> Result<FontEx> {
        // unsafe {
            // let mut info = zeroed();
            // info.cbSize = size_of_val(&info);
            // let res = k32::GetCurrentConsoleFontEx(*self.0, w::FALSE, &mut info);
            // if res == 0 { return last_error() }
            // Ok(FontEx(info))
        // }
    // }
    pub fn write_output(&self, buf: &[CharInfo], size: (i16, i16), pos: (i16, i16)) -> Result<()> {
        assert!(buf.len() == (size.0 as usize) * (size.1 as usize));
        let mut rect = w::SMALL_RECT {
            Left: pos.0,
            Top: pos.1,
            Right: pos.0 + size.0,
            Bottom: pos.1 + size.1,
        };
        let size = w::COORD { X: size.0, Y: size.1 };
        let pos = w::COORD { X: 0, Y: 0 };
        let res = unsafe { k32::WriteConsoleOutputW(
            *self.0, buf.as_ptr() as *const w::CHAR_INFO, size, pos, &mut rect
        )};
        if res == 0 { return last_error() }
        Ok(())
    }
    pub fn font_size(&self) -> Result<(i16, i16)> {
        unsafe {
            let mut font = zeroed();
            let res = k32::GetCurrentConsoleFont(*self.0, w::FALSE, &mut font);
            if res == 0 { return last_error() }
            Ok((font.dwFontSize.X, font.dwFontSize.Y))
        }
    }
}
impl FromRawHandle for ScreenBuffer {
    unsafe fn from_raw_handle(handle: w::HANDLE) -> ScreenBuffer {
        ScreenBuffer(Handle::new(handle))
    }
}
pub struct InputBuffer(Handle);
impl InputBuffer {
    /// Gets the actual active console input buffer
    pub fn from_conin() -> Result<InputBuffer> {
        let handle = unsafe { k32::CreateFileW(
            "CONIN$".to_wide_null().as_ptr(), w::GENERIC_READ | w::GENERIC_WRITE,
            w::FILE_SHARE_READ | w::FILE_SHARE_WRITE, null_mut(), w::OPEN_EXISTING,
            0, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { last_error() }
        else { unsafe { Ok(InputBuffer::from_raw_handle(handle)) } }
    }
    /// The number of input that is available to read
    pub fn available_input(&self) -> Result<u32> {
        let mut num = 0;
        let res = unsafe { k32::GetNumberOfConsoleInputEvents(*self.0, &mut num) };
        if res == 0 { return last_error() }
        Ok(num)
    }
    /// Reads a bunch of input events
    pub fn read_input(&self) -> Result<Vec<Input>> {
        let mut buf: [w::INPUT_RECORD; 0x1000] = unsafe { zeroed() };
        let mut size = 0;
        let res = unsafe { k32::ReadConsoleInputW(
            *self.0, buf.as_mut_ptr(), buf.len() as w::DWORD, &mut size,
        )};
        if res == 0 { return last_error() }
        Ok(buf[..(size as usize)].iter().map(|input| {
            unsafe { match input.EventType {
                w::KEY_EVENT => {
                    let e = input.KeyEvent();
                    Input::Key {
                        key_down: e.bKeyDown != 0,
                        repeat_count: e.wRepeatCount,
                        key_code: e.wVirtualKeyCode,
                        scan_code: e.wVirtualScanCode,
                        wide_char: e.UnicodeChar,
                        control_key_state: e.dwControlKeyState,
                    }
                },
                w::MOUSE_EVENT => {
                    let e = input.MouseEvent();
                    Input::Mouse {
                        position: (e.dwMousePosition.X, e.dwMousePosition.Y),
                        button_state: e.dwButtonState,
                        control_key_state: e.dwControlKeyState,
                        event_flags: e.dwEventFlags,
                    }
                },
                w::WINDOW_BUFFER_SIZE_EVENT => {
                    let s = input.WindowBufferSizeEvent().dwSize;
                    Input::WindowBufferSize(s.X, s.Y)
                },
                w::MENU_EVENT => Input::Menu(input.MenuEvent().dwCommandId),
                w::FOCUS_EVENT => Input::Focus(input.FocusEvent().bSetFocus != 0),
                e => unreachable!("invalid event type: {}", e),
            } }
        }).collect())
    }
    /// Clears all pending input
    pub fn flush_input(&self) -> Result<()> {
        let res = unsafe { k32::FlushConsoleInputBuffer(*self.0) };
        if res == 0 { return last_error() }
        Ok(())
    }
}
impl FromRawHandle for InputBuffer {
    unsafe fn from_raw_handle(handle: w::HANDLE) -> InputBuffer {
        InputBuffer(Handle::from_raw_handle(handle))
    }
}
#[derive(Copy, Clone)]
pub struct ScreenBufferInfo(w::CONSOLE_SCREEN_BUFFER_INFO);
impl ScreenBufferInfo {
    pub fn size(&self) -> (i16, i16) {
        (self.0.dwSize.X, self.0.dwSize.Y)
    }
}
#[derive(Copy, Clone)]
pub struct ScreenBufferInfoEx(w::CONSOLE_SCREEN_BUFFER_INFOEX);
impl ScreenBufferInfoEx {
    pub fn raw_mut(&mut self) -> &mut w::CONSOLE_SCREEN_BUFFER_INFOEX {
        &mut self.0
    }
}
#[derive(Copy, Clone)]
pub struct FontInfoEx(w::CONSOLE_FONT_INFOEX);
#[derive(Copy, Clone)]
pub enum Input {
    Key {
        key_down: bool,
        repeat_count: u16,
        key_code: u16,
        scan_code: u16,
        wide_char: u16,
        control_key_state: u32,
    },
    Mouse {
        position: (i16, i16),
        button_state: u32,
        control_key_state: u32,
        event_flags: u32,
    },
    WindowBufferSize(i16, i16),
    Menu(u32),
    Focus(bool),
}
#[repr(C)] #[derive(Copy, Clone)]
pub struct CharInfo(w::CHAR_INFO);
impl CharInfo {
    pub fn new(ch: u16, attr: u16) -> CharInfo {
        CharInfo(w::CHAR_INFO {
            UnicodeChar: ch,
            Attributes: attr,
        })
    }
    pub fn character(&self) -> u16 { self.0.UnicodeChar }
    pub fn attributes(&self) -> u16 { self.0.Attributes }
}
/// Allocates a console if the process does not already have a console.
pub fn alloc() -> Result<()> {
    match unsafe { k32::AllocConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Detaches the process from its current console.
pub fn free() -> Result<()> {
    match unsafe { k32::FreeConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Attaches the process to the console of the specified process.
/// Pass None to attach to the console of the parent process.
pub fn attach(processid: Option<u32>) -> Result<()> {
    match unsafe { k32::AttachConsole(processid.unwrap_or(-1i32 as u32)) } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Gets the current input code page
pub fn input_code_page() -> u32 {
    unsafe { k32::GetConsoleCP() }
}
/// Gets the current output code page
pub fn output_code_page() -> u32 {
    unsafe { k32::GetConsoleOutputCP() }
}
/// Sets the current input code page
pub fn set_input_code_page(code: u32) -> Result<()> {
    let res = unsafe { k32::SetConsoleCP(code) };
    if res == 0 { return last_error() }
    Ok(())
}
/// Sets the current output code page
pub fn set_output_code_page(code: u32) -> Result<()> {
    let res = unsafe { k32::SetConsoleOutputCP(code) };
    if res == 0 { return last_error() }
    Ok(())
}
