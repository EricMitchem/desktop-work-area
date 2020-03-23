use crate::error::*;

pub struct WorkArea
{
    pub width: i32,
    pub height: i32,
}

pub fn get_work_areas() -> Result<Vec<Result<WorkArea>>>
{
    use std::mem::{MaybeUninit, size_of};
    use std::ptr::null_mut;
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
    use winapi::shared::windef::{HDC, HMONITOR, LPRECT};
    use winapi::um::wingdi::DISPLAY_DEVICEW;
    use winapi::um::wingdi::DISPLAY_DEVICE_ATTACHED_TO_DESKTOP;
    use winapi::um::winuser::EnumDisplayDevicesW;
    use winapi::um::winuser::EnumDisplayMonitors;
    use winapi::um::winuser::GetMonitorInfoW;
    use winapi::um::winuser::MONITORINFOEXW;
    use winapi::um::winuser::LPMONITORINFO;

    unsafe extern "system"
    fn proc(hmonitor: HMONITOR, _hdc: HDC, _lprect: LPRECT, lparam: LPARAM) -> BOOL {
        let results_ptr = lparam as *mut Vec<Result<WorkArea>>;

        let mut monitor_info = MaybeUninit::<MONITORINFOEXW>::zeroed().assume_init();
        monitor_info.cbSize = size_of::<MONITORINFOEXW>() as u32;

        let res = GetMonitorInfoW(hmonitor, &mut monitor_info as *mut MONITORINFOEXW as LPMONITORINFO);

        if res == 0 {
            (*results_ptr).push(Err(Error::expect_winapi_error()));
            return TRUE
        }

        let mut display_device = MaybeUninit::<DISPLAY_DEVICEW>::zeroed().assume_init();
        display_device.cb = size_of::<DISPLAY_DEVICEW>() as u32;

        let res = EnumDisplayDevicesW(monitor_info.szDevice.as_ptr(), 0, &mut display_device as *mut DISPLAY_DEVICEW, 0);

        if res == 0 {
            (*results_ptr).push(Err(Error::expect_winapi_error()));
            return TRUE
        }

        if display_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP != 0 {
            (*results_ptr).push(Ok(WorkArea {
                width: monitor_info.rcWork.right - monitor_info.rcWork.left,
                height: monitor_info.rcWork.bottom - monitor_info.rcWork.top,
            }));
        }

        TRUE
    }

    unsafe {

    let mut results: Vec<Result<WorkArea>> = Vec::new();

    let res = EnumDisplayMonitors(null_mut(), null_mut(), Some(proc), &mut results as *mut Vec<_> as LPARAM);

    match res {
        0 => Err(Error::expect_winapi_error()),
        _ => Ok(results),
    }

    } // unsafe
}