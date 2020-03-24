use std::error;
use std::fmt;
use std::result;

#[derive(Debug)]
pub enum Error
{
    General(String),
    WinApi(String),
}

pub type Result<T> = result::Result<T, Error>;

impl Error
{
    pub fn maybe_winapi_error() -> Option<Error> {
        use std::ptr::null_mut;
        use winapi::um::errhandlingapi::GetLastError;
        use winapi::um::errhandlingapi::SetLastError;
        use winapi::um::winbase::FormatMessageW;
        use winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM;
        use winapi::um::winbase::FORMAT_MESSAGE_IGNORE_INSERTS;

        unsafe {

        let err = GetLastError();

        if err == 0 {
            return None
        }

        let mut buffer: [u16; 1024] = [0; 1024];

        let flags = FORMAT_MESSAGE_FROM_SYSTEM
                  | FORMAT_MESSAGE_IGNORE_INSERTS;

        let res = FormatMessageW(
            flags,
            null_mut(),
            err,
            0,
            buffer.as_mut_ptr(),
            (buffer.len() - 1) as u32,
            null_mut());

        if res == 0 {
            let new_err = GetLastError();
            SetLastError(0);
            return Some(Error::WinApi(
                format!("FormatMessageW failed with error '{}' while handling error '{}'",
                        new_err, err)
            ))
        }

        SetLastError(0);

        let err_slice = &buffer[0..res as usize];
    
        Some(Error::WinApi(
            String::from_utf16_lossy(err_slice)
        ))

        } // unsafe
    }

    pub fn expect_winapi_error() -> Error {
        Error::maybe_winapi_error().unwrap_or_else(||
        Error::into_general_error("Expected winapi error but failed to get it"))
    }

    fn into_general_error<T: Into<String>>(into_string: T) -> Error {
        Error::General(into_string.into())
    }
}

impl error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            General(string) => write!(f, "{}", string),
            WinApi(string) => write!(f, "{}", string),
        }
    }
}