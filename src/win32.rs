use std::iter;

use windows::{
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            DICS_FLAG_GLOBAL, DIF_PROPERTYCHANGE, DIGCF_PRESENT, HDEVINFO, SETUP_DI_STATE_CHANGE,
            SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA, SP_PROPCHANGE_PARAMS,
            SetupDiCallClassInstaller, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
            SetupDiGetClassDevsW, SetupDiGetDeviceInstanceIdW, SetupDiSetClassInstallParamsW,
        },
        Foundation::ERROR_INSUFFICIENT_BUFFER,
        UI::WindowsAndMessaging::{MESSAGEBOX_STYLE, MessageBoxW},
    },
    core::{GUID, PCWSTR, Result, w},
};

pub struct DeviceInfoSet(HDEVINFO);
#[derive(Clone, Copy)]
pub struct DeviceInfoData(SP_DEVINFO_DATA);
#[allow(dead_code)]
pub enum DeviceStateChangeAction {
    Enable = 1,
    Disable = 2,
    Restart = 3, // -> PropChange
}

impl DeviceInfoSet {
    pub unsafe fn new(class_guid: GUID) -> Result<Self> {
        unsafe {
            Ok(Self(SetupDiGetClassDevsW(
                Some(&class_guid),
                None,
                None,
                DIGCF_PRESENT,
            )?))
        }
    }

    pub unsafe fn find_device(&self, instance_id: &'static str) -> Result<Option<DeviceInfoData>> {
        unsafe {
            let devinfo_data_list = self.get_device_info_data_list();
            let index = self.get_index_of_instance(&devinfo_data_list, instance_id)?;
            Ok(index.map(|i| devinfo_data_list[i]))
        }
    }

    pub unsafe fn set_device_state(
        &self,
        devinfo_data: &DeviceInfoData,
        action: DeviceStateChangeAction,
    ) -> Result<()> {
        unsafe {
            let params = SP_PROPCHANGE_PARAMS {
                ClassInstallHeader: SP_CLASSINSTALL_HEADER {
                    cbSize: size_of::<SP_CLASSINSTALL_HEADER>() as u32,
                    InstallFunction: DIF_PROPERTYCHANGE,
                },
                Scope: DICS_FLAG_GLOBAL,
                StateChange: SETUP_DI_STATE_CHANGE(action as u32),
                ..Default::default()
            };
            SetupDiSetClassInstallParamsW(
                self.0,
                Some(&devinfo_data.0),
                Some(&params.ClassInstallHeader),
                size_of::<SP_PROPCHANGE_PARAMS>() as u32,
            )?;
            SetupDiCallClassInstaller(DIF_PROPERTYCHANGE, self.0, Some(&devinfo_data.0))?;
            Ok(())
        }
    }

    unsafe fn get_device_info_data_list(&self) -> Vec<DeviceInfoData> {
        let mut v = Vec::new();
        let mut index = 0;
        loop {
            let mut data = SP_DEVINFO_DATA {
                cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            let result = unsafe { SetupDiEnumDeviceInfo(self.0, index, &mut data) };
            if result.is_err() {
                break;
            }
            v.push(DeviceInfoData(data));
            index += 1;
        }
        v
    }

    unsafe fn get_index_of_instance(
        &self,
        devinfo_data_list: &Vec<DeviceInfoData>,
        instance_id_to_find: &'static str,
    ) -> Result<Option<usize>> {
        for (index, DeviceInfoData(devinfo_data)) in devinfo_data_list.iter().enumerate() {
            unsafe {
                let mut required_size = 0;
                match SetupDiGetDeviceInstanceIdW(
                    self.0,
                    devinfo_data,
                    None,
                    Some(&mut required_size),
                ) {
                    Ok(()) => unreachable!(),
                    Err(err) if err.code() == ERROR_INSUFFICIENT_BUFFER.into() => (),
                    Err(err) => return Err(err),
                }
                let mut buf = vec![0; required_size as usize];
                SetupDiGetDeviceInstanceIdW(self.0, devinfo_data, Some(&mut buf), None)?;
                let device_instance_id: String =
                    char::decode_utf16(buf.into_iter().take(required_size as usize - 1))
                        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                        .collect();
                if instance_id_to_find == device_instance_id {
                    return Ok(Some(index));
                }
            }
        }
        Ok(None)
    }
}

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        unsafe {
            let _ = SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

pub fn show_message_box(text: &str, utype: MESSAGEBOX_STYLE) {
    let text: Vec<_> = text.encode_utf16().chain(iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            None,
            PCWSTR::from_raw(text.as_ptr()),
            w!("surface-wifi-fix"),
            utype,
        );
    }
}
