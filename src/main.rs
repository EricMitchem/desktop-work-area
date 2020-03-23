#[cfg(windows)]
extern crate winapi;

mod error;
mod work_area;

use crate::work_area::*;

fn main()
{
    match get_work_areas() {
        Ok(results) => {
            for res in results.iter() {
                match res {
                    Ok(wa) => {
                        println!("Work area: ({}, {})", wa.width, wa.height)
                    },
                    Err(err) => {
                        println!("Failed to get work area: {}", err)
                    }
                }
            }
        },
        Err(err) => {
            println!("Failed to get work areas: {}", err)
        }
    }
}
