use std::collections::HashMap;

use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    未着手,
    着手中,
    待ち,
    順延,
    完了,
}

struct Logbook {
    frontmatter: Frontmatter,
    lanes: Vec<Lane>,
}

struct Frontmatter {
    date: DateTime<Local>,
    kind: String,
    extra: HashMap<String, String>,
}

pub struct Lane {
    pub title: String,
    pub groups: Vec<Group>,
}

pub struct Group {
    pub heading: String,
    pub items: Vec<Item>,
}

pub struct Item {
    pub id: String,
    pub title: String,
    pub status: Option<Status>,
    pub fields: Vec<(String, String)>,
}
