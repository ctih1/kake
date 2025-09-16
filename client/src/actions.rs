use std::ffi::c_str;

use windows::core::PCSTR;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CAPITAL, VK_F4, VK_L, VK_LWIN, VK_MENU};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONWARNING, MESSAGEBOX_STYLE};

fn create_keypress_pair(key: VIRTUAL_KEY) -> (INPUT, INPUT) {
    unsafe {
        let mut down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: std::mem::zeroed()
        };
    
        let mut up = down.clone();

        down.Anonymous.ki = KEYBDINPUT {
            wVk: key,
            wScan: 0,
            dwFlags: KEYBD_EVENT_FLAGS(0),
            time: 0,
            dwExtraInfo: 0,
        };

        up.Anonymous.ki = KEYBDINPUT {
            wVk: key,
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        return (down,up)
    }

}

pub fn win_lock() {
    unsafe {
        let (l_down, l_up) = create_keypress_pair(VK_L);
        let (win_down, win_up) = create_keypress_pair(VK_LWIN);

        let inputs = &[win_down, l_down,l_up, win_up];
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn dialog(title: String, description: String, style: MESSAGEBOX_STYLE) {
    unsafe {
        MessageBoxA(None,
            PCSTR(std::ffi::CString::new(title).unwrap().as_ptr() as _),
            PCSTR(std::ffi::CString::new(description).unwrap().as_ptr() as _),
            style
        );
    }
}

pub fn alt_f4() {
    unsafe {
        let (f4_down, f4_up) = create_keypress_pair(VK_F4);
        let (alt_down, alt_up) = create_keypress_pair(VK_MENU);

        let inputs = &[alt_down, f4_down, f4_up, alt_up];
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn toggle_caps() {
    unsafe {
        let (caps_down, caps_up) = create_keypress_pair(VK_CAPITAL);
        let inputs = &[caps_down, caps_up];

        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}