extern crate wio;
extern crate winapi;

use wio::{com_ptr_from_fn, com::ComPtr};
use winapi::{
    Class,
    um::{
        combaseapi,
        shobjidl_core::{ShellItem, TaskbarList, IShellItem, ITaskbarList},
    },
};
use std::ptr::null_mut;

#[test]
fn test_multi_com_ptr() {
    unsafe {
        let _: Result<(ComPtr<IShellItem>, ComPtr<ITaskbarList>), _> = com_ptr_from_fn!(
            |(shell_guid, shell_ptr), (taskbar_guid, taskbar_ptr)| {
                let hr = combaseapi::CoCreateInstance(
                    &ShellItem::uuidof(),
                    null_mut(),
                    combaseapi::CLSCTX_ALL,
                    shell_guid,
                    shell_ptr,
                );
                if hr != 0 {
                    return hr;
                }
                combaseapi::CoCreateInstance(
                    &TaskbarList::uuidof(),
                    null_mut(),
                    combaseapi::CLSCTX_ALL,
                    taskbar_guid,
                    taskbar_ptr,
                )
            }
        );
    }
}
