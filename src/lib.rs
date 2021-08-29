pub mod nom;
pub mod utils;

use std::fmt::Display;

#[derive(Default, Debug)]
pub struct YarObj {
    pub name: String,
    tag: String,
    block: String,
}

impl Display for YarObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule {}{} {}", self.name, self.tag, self.block)
    }
}
