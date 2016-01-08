// Copyright © 2016, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
use {IoResult, k32, last_error, w};
use handle::{Handle};
use std::mem::{zeroed};
use std::os::windows::io::{FromRawHandle};
use std::ptr::{null, null_mut};
use wide::{ToWide};

pub struct ScreenBuffer(Handle);
impl ScreenBuffer {
    pub fn new() -> IoResult<ScreenBuffer> {
        let handle = unsafe { k32::CreateConsoleScreenBuffer(
            w::GENERIC_READ | w::GENERIC_WRITE, w::FILE_SHARE_WRITE,
            null(), w::CONSOLE_TEXTMODE_BUFFER, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { last_error() }
        else { unsafe { Ok(ScreenBuffer::from_raw_handle(handle)) } }
    }
    /// Gets the actual active console screen buffer
    pub fn from_stdout() -> IoResult<ScreenBuffer> {
        let handle = unsafe { k32::CreateFileW(
            "CONOUT$".to_wide_null().as_ptr(), w::GENERIC_READ | w::GENERIC_WRITE,
            w::FILE_SHARE_READ, null_mut(), w::OPEN_EXISTING,
            0, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { last_error() }
        else { unsafe { Ok(ScreenBuffer::from_raw_handle(handle)) } }
    }
    /// Gets the actual active console input buffer
    pub fn from_stdin() -> IoResult<ScreenBuffer> {
        let handle = unsafe { k32::CreateFileW(
            "CONIN$".to_wide_null().as_ptr(), w::GENERIC_READ | w::GENERIC_WRITE,
            w::FILE_SHARE_READ | w::FILE_SHARE_WRITE, null_mut(), w::OPEN_EXISTING,
            0, null_mut(),
        )};
        if handle == w::INVALID_HANDLE_VALUE { last_error() }
        else { unsafe { Ok(ScreenBuffer::from_raw_handle(handle)) } }
    }
    pub fn set_active(&self) -> IoResult<()> {
        let res = unsafe { k32::SetConsoleActiveScreenBuffer(*self.0) };
        if res == 0 { last_error() }
        else { Ok(()) }
    }
    pub fn info(&self) -> IoResult<Info> {
        let mut info = Info(unsafe { zeroed() });
        let res = unsafe { k32::GetConsoleScreenBufferInfo(*self.0, &mut info.0) };
        if res == 0 { last_error() }
        else { Ok(info) }
    }
    pub fn set_info_ex(&self) -> IoResult<()> {
        unimplemented!()
    }
    pub fn available_input(&self) -> IoResult<u32> {
        let mut num = 0;
        let res = unsafe { k32::GetNumberOfConsoleInputEvents(*self.0, &mut num) };
        if res == 0 { return last_error() }
        Ok(num)
    }
    pub fn read_input(&self) -> IoResult<Vec<Input>> {
        let mut buf: [w::INPUT_RECORD; 0x1000] = unsafe { zeroed() };
        let mut size = 0;
        let res = unsafe { k32::ReadConsoleInputW(
            *self.0, buf.as_mut_ptr(), buf.len() as w::DWORD, &mut size,
        )};
        if res == 0 { return last_error() }
        Ok(buf[..(size as usize)].iter().map(|input| {
            unsafe { match input.EventType {
                w::KEY_EVENT => Input::Key(*input.KeyEvent()),
                w::MOUSE_EVENT => Input::Mouse(*input.MouseEvent()),
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
    pub fn write_output(&self, buf: &[CharInfo], size: (i16, i16), pos: (i16, i16)) -> IoResult<()> {
        assert!(buf.len() == (size.0 * size.1) as usize);
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
}
impl FromRawHandle for ScreenBuffer {
    unsafe fn from_raw_handle(handle: w::HANDLE) -> ScreenBuffer {
        ScreenBuffer(Handle::from_raw_handle(handle))
    }
}
pub struct Info(w::CONSOLE_SCREEN_BUFFER_INFO);
impl Info {
    pub fn size(&self) -> (i16, i16) {
        (self.0.dwSize.X, self.0.dwSize.Y)
    }
}
#[derive(Debug)]
pub enum Input {
    Key(w::KEY_EVENT_RECORD),
    Mouse(w::MOUSE_EVENT_RECORD),
    WindowBufferSize(i16, i16),
    Menu(u32),
    Focus(bool),
}
#[repr(C)]
pub struct CharInfo(w::CHAR_INFO);
impl CharInfo {
    pub fn new(ch: u16, attr: u16) -> CharInfo {
        CharInfo(w::CHAR_INFO {
            UnicodeChar: ch,
            Attributes: attr,
        })
    }
}
/// Allocates a console if the process does not already have a console.
pub fn alloc() -> IoResult<()> {
    match unsafe { k32::AllocConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Detaches the process from its current console.
pub fn free() -> IoResult<()> {
    match unsafe { k32::FreeConsole() } {
        0 => last_error(),
        _ => Ok(()),
    }
}
/// Attaches the process to the console of the specified process.
/// Pass None to attach to the console of the parent process.
pub fn attach(processid: Option<u32>) -> IoResult<()> {
    match unsafe { k32::AttachConsole(processid.unwrap_or(-1i32 as u32)) } {
        0 => last_error(),
        _ => Ok(()),
    }
}
