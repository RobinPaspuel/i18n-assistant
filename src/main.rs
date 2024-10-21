use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Hello, World!");
    if args.len() > 1 {
        println!("Arguments: {:?}", &args[1..]);
    }
}
