use crate::error::*;
use std::marker::PhantomData;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;

pub struct Window<'a> {
    handle: HWND,
    _marker: PhantomData<&'a WindowClass>,
}

impl<'a> Window<'a> {
    pub fn as_handle(&mut self) -> HWND {
        self.handle
    }

    fn destroy_window(&mut self) {
        use winapi::um::winuser::DestroyWindow;

        unsafe {
            let res = DestroyWindow(self.handle);

            if res == 0 {
                panic!("{}", Error::expect_winapi_error());
            }
        }
    }
}

impl<'a> Drop for Window<'a> {
    fn drop(&mut self) {
        self.destroy_window();
    }
}

pub struct WindowBuilder<'a> {
    class: &'a WindowClass,
    title: Vec<u16>,
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
}

impl<'a> WindowBuilder<'a> {
    pub fn new(class: &'a WindowClass) -> Self {
        WindowBuilder {
            class,
            title: Vec::new(),
            pos_x: 0,
            pos_y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn with_title<T: AsRef<str>>(self, title: T) -> Self {
        let mut title: Vec<u16> = title.as_ref().encode_utf16().collect();
        title.push(0);

        WindowBuilder { title, ..self }
    }

    pub fn with_pos(self, pos_x: i32, pos_y: i32) -> Self {
        WindowBuilder {
            pos_x,
            pos_y,
            ..self
        }
    }

    pub fn with_size(self, width: i32, height: i32) -> Self {
        WindowBuilder {
            width,
            height,
            ..self
        }
    }

    pub fn build(self) -> Result<Window<'a>> {
        use std::ptr::null_mut;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winapi::um::winuser::CreateWindowExW;
        use winapi::um::winuser::{WS_OVERLAPPEDWINDOW, WS_SYSMENU};

        unsafe {
            let handle = CreateWindowExW(
                0,
                self.class.name.as_ptr(),
                self.title.as_ptr(),
                WS_OVERLAPPEDWINDOW & (!WS_SYSMENU),
                self.pos_x,
                self.pos_y,
                self.width,
                self.height,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null_mut()),
                null_mut(),
            );

            if handle == null_mut() {
                return Err(Error::expect_winapi_error());
            }

            Ok(Window {
                handle,
                _marker: PhantomData,
            })
        }
    }
}

pub struct WindowClass {
    name: Vec<u16>,
}

impl WindowClass {
    pub fn new<T: AsRef<str>>(name: T) -> Result<WindowClass> {
        Self::register(name.as_ref())
    }

    fn register(name: &str) -> Result<WindowClass> {
        use std::mem::{size_of, MaybeUninit};
        use std::ptr::null_mut;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winapi::um::winuser::RegisterClassExW;
        use winapi::um::winuser::WNDCLASSEXW;

        unsafe {
            let mut name: Vec<u16> = name.encode_utf16().collect();
            name.push(0);

            let mut wnd_class = MaybeUninit::<WNDCLASSEXW>::zeroed().assume_init();
            wnd_class.cbSize = size_of::<WNDCLASSEXW>() as u32;
            wnd_class.lpfnWndProc = Some(wnd_proc);
            wnd_class.hInstance = GetModuleHandleW(null_mut());
            wnd_class.lpszClassName = name.as_mut_ptr();

            let res = RegisterClassExW(&wnd_class as *const WNDCLASSEXW);

            match res {
                0 => Err(Error::expect_winapi_error()),
                _ => Ok(WindowClass { name }),
            }
        }
    }

    fn unregister(&mut self) {
        use std::ptr::null_mut;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winapi::um::winuser::UnregisterClassW;

        unsafe {
            let res = UnregisterClassW(self.name.as_mut_ptr(), GetModuleHandleW(null_mut()));

            if res == 0 {
                panic!("{}", Error::expect_winapi_error());
            }
        }
    }
}

impl Drop for WindowClass {
    fn drop(&mut self) {
        self.unregister();
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use winapi::um::winuser::{DefWindowProcW, PostQuitMessage, WM_DESTROY};

    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return 0;
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
