use std::fmt::{ Display, Formatter };
use regex::Regex;
use crate::parser::Stmt;

pub struct Icon {
    icon: char,
    regex: Regex,
    reversed: bool
}

impl Display for Icon {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.icon)
    }
}

impl Icon {
    pub fn new(icon: char, regex: &str, reversed: bool) -> Result<Self, std::io::Error> {
        let regex = match Regex::new(&format!("^{}$", regex)) {
            Ok(r) => r,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        };

        Ok(Self { icon, regex, reversed })
    }

    pub fn matches(&self, str: &str) -> bool {
        self.regex.is_match(str)
    }

    pub fn reversed_class(&self) -> char {
        if self.reversed { return 'r' }
        return ' '
    }
}

pub struct Icons {
    empty: Icon,
    default: Icon,
    icons: Vec<Stmt>
}

impl Icons {
    pub fn new() -> Self {
        Self { 
            empty: Icon::new(' ', " ", false).unwrap(),
            default: Icon::new(' ', " ", false).unwrap(),
            icons: vec![],
        }
    }

    pub fn set_icon(&mut self, icon_stmt: Stmt) {
        self.icons.push(icon_stmt);
    }


    pub fn set_default(&mut self, icon: Icon) {
        self.default = icon;
    }

    pub fn set_empty(&mut self, icon: Icon) {
        self.empty = icon;
    }

    pub fn get_icon(&self, class: &str, title: &str) -> Option<&Icon> {
        for stmt in &self.icons {
            match stmt {
                Stmt::Class(i) => if i.matches(class) { return Some(i); }
                Stmt::Title(i) => if i.matches(title) { return Some(i); }
                _ => continue,
            }
        }
        
        None
    }

    pub fn get_default(&self) -> &Icon {
        &self.default
    }

    pub fn get_empty(&self) -> &Icon {
        &self.empty
    }
}



