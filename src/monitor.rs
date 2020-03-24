use std::fmt;
use crate::error::*;

pub struct MonitorArea
{
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct MonitorInfo
{
    pub name: String,
    pub area: MonitorArea,
    pub work_area: MonitorArea,
}

impl fmt::Display for MonitorInfo
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let monitor_info = format!("\
            {separator}\n\
            Name: {name}\n\
            Origin: ({area_x}, {area_y})\n\
            Size: ({area_w}, {area_h})\n\
            Work area origin: ({work_area_x}, {work_area_y})\n\
            Work area size: ({work_area_w}, {work_area_h})\n\
            {separator}",
            separator = "-".repeat(40),
            name = self.name,
            area_x = self.area.x,
            area_y = self.area.y,
            area_w = self.area.width,
            area_h = self.area.height,
            work_area_x = self.work_area.x,
            work_area_y = self.work_area.y,
            work_area_w = self.work_area.width,
            work_area_h = self.work_area.height,
        );

        write!(f, "{}", monitor_info)
    }
}

pub fn  query_monitors() -> Result<Vec<Result<MonitorInfo>>>
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
        let results_ptr = lparam as *mut Vec<Result<MonitorInfo>>;

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
            let name = String::from_utf16_lossy(monitor_info.szDevice.split(|&e| e == 0).next().unwrap());

            let area = MonitorArea {
                x: monitor_info.rcMonitor.left,
                y: monitor_info.rcMonitor.top,
                width: monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                height: monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
            };

            let work_area = MonitorArea {
                x: monitor_info.rcWork.left,
                y: monitor_info.rcWork.top,
                width: monitor_info.rcWork.right - monitor_info.rcWork.left,
                height: monitor_info.rcWork.bottom - monitor_info.rcWork.top,
            };

            (*results_ptr).push(Ok(MonitorInfo {
                name,
                area,
                work_area,
            }));
        }

        TRUE
    }

    unsafe {

    let mut results: Vec<Result<MonitorInfo>> = Vec::new();

    let res = EnumDisplayMonitors(null_mut(), null_mut(), Some(proc), &mut results as *mut Vec<_> as LPARAM);

    match res {
        0 => Err(Error::expect_winapi_error()),
        _ => Ok(results),
    }

    } // unsafe
}