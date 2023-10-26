extern crate embed_resource;

use embed_resource::{compile, NONE};

fn main() {
    compile("tray-example.rc", NONE);
}
