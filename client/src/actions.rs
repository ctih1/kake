use std::mem::zeroed;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Interface, PCSTR, PCWSTR};
use windows::Win32::Graphics::Gdi::{CDS_TYPE, ChangeDisplaySettingsExA, DEVMODE_DISPLAY_ORIENTATION, DEVMODEA, DISP_CHANGE_SUCCESSFUL, DISPLAY_DEVICEA, DM_DISPLAYORIENTATION, DM_PELSHEIGHT, DM_PELSWIDTH, DMDO_90, DMDO_270, EnumDisplayDevicesA};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback};
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceActivator, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::System::Diagnostics::Debug::Beep;
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, LoadKeyboardLayoutA, SendInput, VIRTUAL_KEY, VK_CAPITAL, VK_F4, VK_L, VK_LWIN, VK_MENU, VK_SPACE, VkKeyScanExA};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, EDD_GET_DEVICE_INTERFACE_NAME, MB_ICONWARNING, MESSAGEBOX_STYLE};
use windows::Win32::System::Shutdown::{LockWorkStation, SHTDN_REASON_FLAG_USER_DEFINED};
use windows::Win32::System::Shutdown::InitiateSystemShutdownExA;
use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper};
use std::sync::{Arc, Mutex};

use crate::logs;

pub struct Actions {
    pub logger: Arc<Mutex<logs::DebugLogs>>
}

impl Actions {
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

    pub fn lock() {
        unsafe {
            LockWorkStation();
        }
    }

    pub fn dialog(&mut self, title: String, description: String, style: MESSAGEBOX_STYLE) {
        unsafe {
            let result = MessageBoxA(None,
                PCSTR(std::ffi::CString::new(description).unwrap().as_ptr() as _),
                PCSTR(std::ffi::CString::new(title).unwrap().as_ptr() as _),
                style
            );

            if result.0 == 0 {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to show textbot (error 0)"));
            }
        }
    }

    pub fn shutdown(&mut self) {
        unsafe {
            let result = InitiateSystemShutdownExA(None, None, 2 as u32, false, true, SHTDN_REASON_FLAG_USER_DEFINED);

            if result.is_err() {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to shut down"));
            }
        }
    }

    pub fn beep(&mut self, duration_ms: u32) {
        unsafe {
            // From personal testing, I found out that 4000hz is the worst frequency out there
            let result = Beep(4000 as u32, duration_ms);

            if result.is_err() {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to beep"));
            }
        }
    }

    pub fn rotate_monitor(&mut self, orientation: DEVMODE_DISPLAY_ORIENTATION) {
        unsafe {
            let mut devmode: DEVMODEA = zeroed();
            devmode.dmSize = std::mem::size_of::<DEVMODEA>() as u16;
            devmode.dmFields = DM_DISPLAYORIENTATION;
            devmode.Anonymous1.Anonymous2.dmDisplayOrientation = orientation;
            let mut display_device: DISPLAY_DEVICEA = zeroed();
            display_device.cb = std::mem::size_of::<DISPLAY_DEVICEA>() as u32;
            EnumDisplayDevicesA(None, 0 as u32, &mut display_device, EDD_GET_DEVICE_INTERFACE_NAME).unwrap();        
            let status = ChangeDisplaySettingsExA(PCSTR::from_raw(display_device.DeviceName.as_ptr() as _), Some(&mut devmode), None, CDS_TYPE(0), None);

            if status != DISP_CHANGE_SUCCESSFUL {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to rotate display") + &status.0.to_string());
            }
        }
    }

    pub fn set_volume(&mut self, volume_percentage: u8) {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator  = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap();
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).unwrap();
            let result = endpoint_volume.GetMute();
            if result.is_err() {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to get volume muted status"));
            }
            if let Ok(muted) = result {
                if muted.as_bool() {
                    let result = endpoint_volume.SetMute(false, std::ptr::null());
                    if result.is_err() {
                        let mut logger = self.logger.lock().unwrap();
                        logger.error(lc!("Failed to unmute"));
                    }
                }
            }
        }
    }

    pub fn alt_f4() {
        unsafe {
            let (f4_down, f4_up) = Self::create_keypress_pair(VK_F4);
            let (alt_down, alt_up) = Self::create_keypress_pair(VK_MENU);

            let inputs = &[alt_down, f4_down, f4_up, alt_up];
            SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }


    pub fn space_bar() {
        unsafe {
            let (space_down, space_up) = Self::create_keypress_pair(VK_SPACE);

            let inputs = &[space_down, space_up];
            SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn toggle_caps() {
        unsafe {
            let (caps_down, caps_up) = Self::create_keypress_pair(VK_CAPITAL);
            let inputs = &[caps_down, caps_up];

            SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn change_wallpaper(&mut self, path: String) {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let wallpaper: IDesktopWallpaper  = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL).unwrap();
            let result = wallpaper.SetWallpaper(None, PCWSTR(std::ffi::OsStr::new(&path).encode_wide().chain(Some(0)).collect::<Vec<u16>>().as_ptr()));

            if result.is_err() {
                let mut logger = self.logger.lock().unwrap();
                logger.error(lc!("Failed to change wallpaper ") + &result.err().unwrap().message());
            }
        }
    }
}
