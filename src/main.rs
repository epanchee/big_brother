mod slaves;

use crate::slaves::*;

fn main() {
    // fetchers::main();
    // config_parser::serialize();
    println!("{:#?}", config_parser::parse_yaml("configs/example.yaml"));
}
