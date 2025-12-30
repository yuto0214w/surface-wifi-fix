#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use windows::core::{GUID, Result};

mod win32;
use win32::*;

fn main() -> Result<ExitCode> {
    let class_guid = GUID::from_u128(0x4d36e972_e325_11ce_bfc1_08002be10318);
    let instance_id = "PCI\\VEN_11AB&DEV_2B38&SUBSYS_045E0008";
    unsafe {
        let dev_info_set = DeviceInfoSet::new(class_guid)?;
        let dev_info_data = match dev_info_set.find_device(instance_id)? {
            Some(data) => data,
            None => {
                eprintln!("Could not find device");
                return Ok(ExitCode::FAILURE);
            }
        };
        dev_info_set.set_device_state(&dev_info_data, DeviceStateChangeAction::Restart)?;
        // dev_info_set.set_device_state(&dev_info_data, false)?;
        // dev_info_set.set_device_state(&dev_info_data, true)?;
    }
    Ok(ExitCode::SUCCESS)
}
