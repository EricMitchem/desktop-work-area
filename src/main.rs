#[cfg(windows)]
extern crate winapi;

mod error;
mod monitor;

use crate::monitor::*;

fn main()
{
    match query_monitors() {
        Ok(results) => {
            for res in results.iter() {
                match res {
                    Ok(info) => {
                        println!("{}", info);
                    },
                    Err(err) => {
                        println!("Failed to get monitor info: {}", err);
                    }
                }
            }
        },
        Err(err) => {
            println!("Failed to query monitors: {}", err);
        }
    }
}
