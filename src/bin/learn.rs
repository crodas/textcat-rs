use std::env;
use textcat::category::learn_from_directory;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _p = learn_from_directory(&args[1])
        .unwrap()
        .persist(&args[2])
        .unwrap();

    println!("{} has been created", &args[2]);
}
