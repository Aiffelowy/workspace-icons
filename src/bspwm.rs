use xcb::x;
use xcb::XidNew;
use bspc_rs::events::{subscribe, DesktopEvent, Event, NodeEvent, Subscription};
use bspc_rs::selectors::{DesktopSelector, NodeSelector};
use std::collections::BTreeMap;
use crate::icons::Icons;
use std::sync::{ Mutex, Arc };

xcb::atoms_struct! {
    #[derive(Copy, Clone, Debug)]
    pub(crate) struct Atoms {
        pub wm_name => b"_NET_WM_NAME" only_if_exists = false,
    }
}

#[derive(Debug)]
struct KnownWindow {
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
}


fn print_icons(windows: &BTreeMap<u32, Option<KnownWindow>>, focused :&u32, icons: &Icons) {
    let mut string :String = String::from("(eventbox :onscroll \"sh $HOME/.config/nixos/home/shared/dotfiles/assets/eww/scripts/dashActions.sh '{}'\" :valign \"center\" (box	:class \"ws\" :orientation \"h\"	:halign \"center\"	:valign \"center\"	 :space-evenly \"true\" ");

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

        string += &format!("(button :onclick \"bspc desktop -f {desktop}\"	:class	\"{} {} {}\" \"{icon}\")",
                if desktop == focused { "focused" } else { "" },
                if window.is_some() { "occupied" } else { "" },
                icon.reversed_class());
    }

    println!("{string}))");
}



pub fn thread_bspwm(icons: Arc<Mutex<Icons>>, args: Vec<String>) -> xcb::Result<()> {
    let (x_conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = x_conn.get_setup();
    let _screen = setup.roots().nth(screen_num as usize).unwrap();
    let atoms = Atoms::intern_all(&x_conn)?;
    

    let subscriptions = vec![ Subscription::NodeTransfer, Subscription::NodeFocus, Subscription::NodeRemove, Subscription::NodeAdd, Subscription::DesktopFocus ];
    let mut subscriber = subscribe(false, None, &subscriptions).unwrap();

    let mut windows :BTreeMap<u32, Option<KnownWindow>> = BTreeMap::new();
    let mut focused_desktop :u32 = 0;

    for workspace in args {
        if let Ok(desktop_id) = bspc_rs::query::query_desktops(false, None, None, Some(DesktopSelector(&workspace)), None) {
            focused_desktop = desktop_id[0];
            if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&workspace)), None) {
                let kw = KnownWindow::new(&x_conn, &atoms, &nodes[0]);
                windows.insert(desktop_id[0], Some(kw));
            } else {
                windows.insert(desktop_id[0], None);
            }
        }
    }

    print_icons(&windows, &focused_desktop, &icons.lock().expect("Failed to aquire lock"));

    macro_rules! skip_workspaces {
        ($w: expr) => {
            if !windows.contains_key(&$w) { continue; }
        };
    }

    for event in subscriber.events() {
        match event.unwrap() {
            Event::NodeEvent(event) => match event {
                NodeEvent::NodeTransfer(node_info) => {
                    if windows.contains_key(&node_info.src_desktop_id) {
                        if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&format!("{}", node_info.src_desktop_id))), None) {
                            windows.insert(node_info.src_desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &nodes[0])));
                        } else {
                            windows.insert(node_info.src_desktop_id, None);
                        }
                    }
                    
                    skip_workspaces!(node_info.dst_desktop_id);
                    windows.insert(node_info.dst_desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.src_node_id)));
                }

                NodeEvent::NodeAdd(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    windows.insert(node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.node_id)));
                }

                NodeEvent::NodeFocus(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    windows.insert(node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.node_id)));

                }

                NodeEvent::NodeRemove(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&format!("{}", node_info.desktop_id))), None) {
                        windows.insert(node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &nodes[0])));
                    } else {
                        windows.insert(node_info.desktop_id, None);
                    }
                }

                _ => unreachable!()
            }

            Event::DesktopEvent(event) => {
                match event {
                    DesktopEvent::DesktopFocus(desktop_info) => { skip_workspaces!(desktop_info.desktop_id); focused_desktop = desktop_info.desktop_id },
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }

       print_icons(&windows, &focused_desktop, &icons.lock().unwrap());
    }

    Ok(())

}

