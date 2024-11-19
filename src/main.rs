use std::sync::{ Mutex, Arc };
use std::env::args;
use std::thread;


pub mod parser;
pub mod config;
pub mod bspwm;
pub mod icons;
pub mod window;
//pub mod any_wm;


use crate::icons::Icons;
use crate::bspwm::thread_bspwm;
//use any_wm::thread_any_wm;
use crate::config::thread_config;



fn main() -> xcb::Result<()> {
    let icons :Arc<Mutex<Icons>> = Arc::new(Mutex::new(Icons::new()));
    let mut args = args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => { println!("Path to config not given"); return Ok(()) }
    };

    let workspaces :Vec<String> = args.collect();

    let icons_arc = icons.clone();
    let config_thread_handle = thread::spawn(move || { match thread_config(icons_arc.clone(), &path) {
        Ok(_) => (),
        Err(e) => println!("config error: {:?}", e)
    }});

    let icons_arc = icons.clone();
    let bspwm_thread_handle = thread::spawn(move || { match thread_bspwm(icons_arc.clone(), workspaces) {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e)
    }});

/*    let bspwm_thread_handle = thread::spawn(move || { match thread_any_wm(icons_arc.clone(), workspaces) {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e),
    }});
*/
    match config_thread_handle.join() {
        Ok(_) => (),
        Err(e) => println!("config error: {:?}", e),
    }

    match bspwm_thread_handle.join() {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e),
    }

    Ok(())
}
