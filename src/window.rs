use xcb::x;
use xcb::XidNew;
use std::collections::BTreeMap;
use crate::icons::Icons;
use strfmt::{strfmt, strfmt_builder};

xcb::atoms_struct! {
    #[derive(Copy, Clone, Debug)]
    pub struct Atoms {
        pub wm_name => b"_NET_WM_NAME" only_if_exists = false,
    }
}

#[derive(Debug)]
pub struct KnownWindow {
    pub class: String,
    pub title: String
}

impl KnownWindow {
    pub fn new(x_conn: &xcb::Connection, atoms: &Atoms, window_id: &u32) -> Self {
        let window :x::Window;
        unsafe { window = x::Window::new(*window_id); } //i never had this error out so its fine, right?
        let cookie_class = x_conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: x::ATOM_WM_CLASS,
            r#type: x::ATOM_STRING,
            long_offset: 0,
            long_length: 8
        });

        let cookie_title = x_conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: atoms.wm_name,
            r#type: x::ATOM_ANY,
            long_offset: 0,
            long_length: 32
        });

        let class = match x_conn.wait_for_reply(cookie_class) {
            Ok(r) => std::str::from_utf8(r.value()).expect("The WM_CLASS property is not valid UTF-8").split("\0").nth(1).unwrap_or("sus").to_string(),
            Err(_) => "who knows?".to_string()
        };

        let title = match x_conn.wait_for_reply(cookie_title) {
            Ok(r) => { std::str::from_utf8(r.value()).expect("The _NET_WM_NAME property is not valid UTF-8").to_string()},
            Err(_) => "who knows?".to_string()
        };

        Self { class, title }
    }

    pub fn new_known(class :String, title: String) -> Self {
        Self { class, title }
    }
}


pub fn print_icons(windows: &BTreeMap<String, Option<KnownWindow>>, focused :&str, icons: &Icons) {
    let mut string :String = icons.get_before().to_string();

    for (desktop, window) in windows {
        let icon = match window {
            Some(w) => {
                match icons.get_icon(&w.class, &w.title) {
                    Some(t) => t,
                    None => icons.get_default()
                }
            }

            None => icons.get_empty()
        };

        //bc the crate is cool but also kinda bad
        let reversed_str = icon.reversed_class().to_string();
        let mut color = match &icon.color {
            Some(c) => c.to_string(),
            None => icons.get_default().color.as_ref().unwrap().to_string(),
        };

        let mut fcolor = match &icon.fcolor {
            Some(c) => c.to_string(),
            None => icons.get_default().fcolor.as_ref().unwrap().to_string(),
        };

        let class = match window {
            Some(w) => w.class.clone(),
            None => "".to_string()
        };

        if icon.reversed_class() == 'r' {
            std::mem::swap(&mut color, &mut fcolor);
        }

        string += &format!("{}", strfmt!(icons.get_fmt(),
                desktop => desktop.to_string(),
                icon => icon.to_string(),
                focused => if *desktop == focused { "focused" } else { "" },
                occupied => if window.is_some() { "occupied" } else { "" },
                color => if *desktop == focused { fcolor } else { color },
                window_class => class,
                reversed => reversed_str).unwrap());
    }

    println!("{string}{}", icons.get_after());
}

