use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use tera::{Context, Tera};
use textcat::storage::learn_from_directory;

fn main() {
    let _p = learn_from_directory("samples").unwrap();
    let mut tera = Tera::default();

    let mut f = File::open("src/template.rs").unwrap();
    let mut code = String::new();
    f.read_to_string(&mut code).unwrap();

    tera.add_raw_template("embed", &code).unwrap();

    let mut context = Context::new();
    context.insert("ngrams", &_p.to_vec());
    context.insert("languages", &_p.categories());
    context.insert("version", &env!("CARGO_PKG_VERSION").to_string());

    File::create("../src/default.rs")
        .unwrap()
        .write_all(tera.render("embed", &context).unwrap().as_bytes())
        .unwrap();

    Command::new("cargo")
        .arg("fmt")
        .arg("--manifest-path")
        .arg("../Cargo.toml")
        .output()
        .unwrap();
}
