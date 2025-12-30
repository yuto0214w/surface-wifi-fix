#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use windows::{
    Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK},
    core::{GUID, Result},
};

mod win32;
use win32::*;

fn main() -> Result<ExitCode> {
    let class_guid = GUID::from_u128(0x4d36e972_e325_11ce_bfc1_08002be10318);
    let instance_id = "PCI\\VEN_11AB&DEV_2B38&SUBSYS_045E0008&REV_00\\4&22AAFC1D&0&00E0";

    let result: Result<ExitCode> = (|| unsafe {
        let dev_info_set = DeviceInfoSet::new(class_guid)?;
        let dev_info_data = match dev_info_set.find_device(instance_id)? {
            Some(data) => data,
            None => {
                show_message_box("Could not find device", MB_ICONERROR | MB_OK);
                return Ok(ExitCode::FAILURE);
            }
        };
        dev_info_set.set_device_state(&dev_info_data, DeviceStateChangeAction::Restart)?;
        // dev_info_set.set_device_state(&dev_info_data, false)?;
        // dev_info_set.set_device_state(&dev_info_data, true)?;

        show_message_box("Restarted successfully", MB_OK);
        Ok(ExitCode::SUCCESS)
    })();

    match result {
        Err(err) => {
            show_message_box(
                &format!("An error has occurred:\r\n\r\n{}", err.message()),
                MB_ICONERROR | MB_OK,
            );
            return Err(err);
        }
        ok => ok,
    }
}
