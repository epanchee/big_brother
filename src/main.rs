mod slaves;

use crate::slaves::*;

fn main() {
    // fetchers::main();
    // println!("{:#?}", config_parser::parse_yaml("configs/example.yaml"));
    println!("{:#?}", config_parser::parse_config_dir("configs"));
}
