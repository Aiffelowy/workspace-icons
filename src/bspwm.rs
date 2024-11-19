use bspc_rs::events::{subscribe, DesktopEvent, Event, NodeEvent, Subscription};
use bspc_rs::selectors::{DesktopSelector, NodeSelector};
use std::sync::{ Mutex, Arc };
use crate::icons::Icons;
use crate::window::{Atoms, KnownWindow, print_icons};
use std::collections::{ BTreeMap, HashMap };

pub fn thread_bspwm(icons: Arc<Mutex<Icons>>, args: Vec<String>) -> xcb::Result<()> {
    let (x_conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = x_conn.get_setup();
    let _screen = setup.roots().nth(screen_num as usize).unwrap();
    let atoms = Atoms::intern_all(&x_conn)?;
    

    let subscriptions = vec![ Subscription::NodeTransfer, Subscription::NodeFocus, Subscription::NodeRemove, Subscription::NodeAdd, Subscription::DesktopFocus ];
    let mut subscriber = subscribe(false, None, &subscriptions).unwrap();

    let mut desktops :HashMap<u32, String> = HashMap::new();
    let mut windows :BTreeMap<String, Option<KnownWindow>> = BTreeMap::new();
    let mut focused_desktop :String = String::new();

    for workspace in args {
        if let Ok(desktop_id) = bspc_rs::query::query_desktops(false, None, None, Some(DesktopSelector(&workspace)), None) {
            focused_desktop = workspace.clone();
            desktops.insert(desktop_id[0], workspace.clone());
            if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&workspace)), None) {
                let kw = KnownWindow::new(&x_conn, &atoms, &nodes[0]);
                windows.insert(workspace, Some(kw));
            } else {
                windows.insert(workspace, None);
            }
        }
    }

    print_icons(&windows, &focused_desktop, &icons.lock().expect("Failed to aquire lock"));

    macro_rules! skip_workspaces {
        ($w: expr) => {
            if !desktops.contains_key(&$w) { continue; }
        };
    }

    macro_rules! change_window {
        ($id: expr, $window: expr) => {
            windows.insert(desktops.get($id).unwrap().to_string(), $window)
        };
    }

    for event in subscriber.events() {
        match event.unwrap() {
            Event::NodeEvent(event) => match event {
                NodeEvent::NodeTransfer(node_info) => {
                    if desktops.contains_key(&node_info.src_desktop_id) {
                        if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&format!("{}", node_info.src_desktop_id))), None) {
                            change_window!(&node_info.src_desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &nodes[0])));
                        } else {
                            change_window!(&node_info.src_desktop_id, None);
                        }
                    }
                    
                    skip_workspaces!(node_info.dst_desktop_id);
                    change_window!(&node_info.dst_desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.src_node_id)));
                }

                NodeEvent::NodeAdd(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    change_window!(&node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.node_id)));
                }

                NodeEvent::NodeFocus(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    change_window!(&node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &node_info.node_id)));

                }

                NodeEvent::NodeRemove(node_info) => {
                    skip_workspaces!(node_info.desktop_id);
                    if let Ok(nodes) = bspc_rs::query::query_nodes(Some(NodeSelector(".window")), None, Some(DesktopSelector(&format!("{}", node_info.desktop_id))), None) {
                        change_window!(&node_info.desktop_id, Some(KnownWindow::new(&x_conn, &atoms, &nodes[0])));
                    } else {
                        change_window!(&node_info.desktop_id, None);
                    }
                }

                _ => unreachable!()
            }

            Event::DesktopEvent(event) => {
                match event {
                    DesktopEvent::DesktopFocus(desktop_info) => { skip_workspaces!(desktop_info.desktop_id); focused_desktop = desktops.get(&desktop_info.desktop_id).unwrap().to_string() },
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }

       print_icons(&windows, &focused_desktop, &icons.lock().unwrap());
    }

    Ok(())

}

