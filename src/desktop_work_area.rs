use crate::error::*;

pub struct DesktopWorkArea
{
    pub width: i32,
    pub height: i32,
}

pub fn get_desktop_work_area() -> Result<DesktopWorkArea>
{
    use std::mem::MaybeUninit;
    use winapi::shared::windef::RECT;
    use winapi::um::winnt::PVOID;
    use winapi::um::winuser::{SystemParametersInfoW, SPI_GETWORKAREA};

    unsafe {

    let mut rect = MaybeUninit::<RECT>::uninit();
    let res = SystemParametersInfoW(SPI_GETWORKAREA, 0, rect.as_mut_ptr() as PVOID, 0);

    if res == 0 {
        return Err(Error::expect_winapi_error())
    }

    let rect = rect.assume_init();

    Ok(DesktopWorkArea {
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    })

    } // unsafe
}