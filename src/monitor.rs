use crate::error::*;
use std::fmt;
use std::mem::{size_of, MaybeUninit};
use std::ptr::null_mut;
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::{HDC, HMONITOR, LPRECT};

pub struct MonitorArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct MonitorInfo {
    pub name: String,
    pub area: MonitorArea,
    pub work_area: MonitorArea,
    pub client_area: MonitorArea,
}

impl fmt::Display for MonitorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let monitor_info = format!(
            "\
            {separator}\n\
            Name: {name}\n\
            Origin: ({area_x}, {area_y})\n\
            Size: ({area_w}, {area_h})\n\
            Work area origin: ({work_area_x}, {work_area_y})\n\
            Work area size: ({work_area_w}, {work_area_h})\n\
            Window client area size: ({client_area_w}, {client_area_h})\n\
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
            client_area_w = self.client_area.width,
            client_area_h = self.client_area.height,
        );

        write!(f, "{}", monitor_info)
    }
}

pub fn query_monitors() -> Result<Vec<Result<MonitorInfo>>> {
    use winapi::um::winuser::EnumDisplayMonitors;

    unsafe {
        let mut results: Vec<Result<MonitorInfo>> = Vec::new();

        let res = EnumDisplayMonitors(
            null_mut(),
            null_mut(),
            Some(monitor_enum_proc),
            &mut results as *mut Vec<_> as LPARAM,
        );

        match res {
            0 => Err(Error::expect_winapi_error()),
            _ => Ok(results),
        }
    }
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: LPRECT,
    lparam: LPARAM,
) -> BOOL {
    use winapi::shared::minwindef::TRUE;
    use winapi::um::wingdi::DISPLAY_DEVICEW;
    use winapi::um::wingdi::DISPLAY_DEVICE_ATTACHED_TO_DESKTOP;
    use winapi::um::winuser::EnumDisplayDevicesW;
    use winapi::um::winuser::GetMonitorInfoW;
    use winapi::um::winuser::LPMONITORINFO;
    use winapi::um::winuser::MONITORINFOEXW;

    let results_ptr = lparam as *mut Vec<Result<MonitorInfo>>;

    let mut monitor_info = MaybeUninit::<MONITORINFOEXW>::zeroed().assume_init();
    monitor_info.cbSize = size_of::<MONITORINFOEXW>() as u32;

    let res = GetMonitorInfoW(
        hmonitor,
        &mut monitor_info as *mut MONITORINFOEXW as LPMONITORINFO,
    );

    if res == 0 {
        (*results_ptr).push(Err(Error::expect_winapi_error()));
        return TRUE;
    }

    let mut display_device = MaybeUninit::<DISPLAY_DEVICEW>::zeroed().assume_init();
    display_device.cb = size_of::<DISPLAY_DEVICEW>() as u32;

    let res = EnumDisplayDevicesW(
        monitor_info.szDevice.as_ptr(),
        0,
        &mut display_device as *mut DISPLAY_DEVICEW,
        0,
    );

    if res == 0 {
        (*results_ptr).push(Err(Error::expect_winapi_error()));
        return TRUE;
    }

    if display_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP != 0 {
        let name =
            String::from_utf16_lossy(monitor_info.szDevice.split(|&e| e == 0).next().unwrap());

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

        let client_area = match query_client_area(&work_area) {
            Err(err) => {
                (*results_ptr).push(Err(err));
                return TRUE;
            }
            Ok(client_area) => client_area,
        };

        (*results_ptr).push(Ok(MonitorInfo {
            name,
            area,
            work_area,
            client_area,
        }));
    }

    TRUE
}

unsafe fn query_client_area(work_area: &MonitorArea) -> Result<MonitorArea> {
    use crate::window::*;
    use winapi::shared::windef::RECT;
    use winapi::um::winuser::TranslateMessage;
    use winapi::um::winuser::{DispatchMessageW, GetClientRect};
    use winapi::um::winuser::{GetMessageW, MSG};

    let window_class = match WindowClass::new("desktop-work-area-wc") {
        Err(err) => return Err(err),
        Ok(wc) => wc,
    };

    let window_builder = WindowBuilder::new(&window_class)
        .with_title("desktop-work-area")
        .with_pos(work_area.x, work_area.y)
        .with_size(work_area.width, work_area.height);

    let client_area = {
        let mut window = match window_builder.build() {
            Err(err) => return Err(err),
            Ok(window) => window,
        };

        let mut rect = MaybeUninit::<RECT>::uninit();

        let res = GetClientRect(window.as_handle(), rect.as_mut_ptr());

        if res == 0 {
            return Err(Error::expect_winapi_error());
        }

        let rect = rect.assume_init();

        MonitorArea {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    };

    let mut msg = MaybeUninit::<MSG>::uninit().assume_init();

    loop {
        match GetMessageW(&mut msg as *mut MSG, null_mut(), 0, 0) {
            -1 => return Err(Error::expect_winapi_error()),
            0 => break,
            _ => {
                TranslateMessage(&msg as *const MSG);
                DispatchMessageW(&msg as *const MSG);
            }
        }
    }

    Ok(client_area)
}
