#[cfg(windows)]
extern crate winapi;

mod desktop_work_area;
mod error;

use crate::desktop_work_area::*;

fn main() {
    match get_desktop_work_area() {
        Ok(dwa) => {
            println!("Desktop work area: ({}, {})", dwa.width, dwa.height)
        },
        Err(err) => {
            println!("Failed to get desktop work area: {}", err)
        }
    }
}
