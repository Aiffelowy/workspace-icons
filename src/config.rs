use crate::parser::{Lexer, Parser, Stmt};
use crate::icons::Icons;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::sync::{ Mutex, Arc, mpsc };

fn read_config(icons: &mut Icons, path: &str) -> Result<(), std::io::Error> {
    let config = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "error reading file"))
    };

    let lexer = Lexer::new();
    let mut parser = Parser::new(lexer);

    *icons = Icons::new();
    for line in config.lines() {
        parser.feed_next_line(line);
        let stmt = parser.parse()?;
        match stmt {
            Stmt::Default(i) => icons.set_default(i),
            Stmt::Empty(i) => icons.set_empty(i),
            Stmt::FmtBefore(f) => icons.set_before(f),
            Stmt::Fmt(f) => icons.set_fmt(f),
            Stmt::FmtAfter(f) => icons.set_after(f),
            Stmt::None => continue,
            _ => icons.set_icon(stmt),
        }
    }

    Ok(())
}

pub fn thread_config(icons_arc: Arc<Mutex<Icons>>, path: &str) -> Result<(), std::io::Error> {
    if let Ok(mut icons) = icons_arc.lock() {
        read_config(&mut icons, path)?;
    }

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = match recommended_watcher(tx) {
        Ok(w) => w,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
    };

    match watcher.watch(std::path::Path::new(path), RecursiveMode::Recursive) {
        Ok(()) => (),
        Err(e) =>  return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    for res in rx {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Modify(_) => {
                    if let Ok(mut icons) = icons_arc.lock() {
                        let _ = read_config(&mut icons, path); //we let this error bc something something nvim does funny shit with files and it doesnt work
                    }
                },
                notify::EventKind::Remove(_) => { // again bc nvim like, removes the files after modify? but its there? and it fuckes up everything
                    if let Ok(mut icons) = icons_arc.lock() {
                        read_config(&mut icons, path)?;
                        match watcher.watch(std::path::Path::new(path), RecursiveMode::Recursive) {
                            Ok(()) => (),
                            Err(e) =>  return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                        }

                    }
                },
                _ => (),
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
