use libelp::Configuration;
use libelp::my_vec;
#[allow(dead_code)]
#[derive(Configuration)]
struct Test {
    #[default = "8080"]
    #[note = "The port number of the server"]
    age: u16,
}

fn main() {
    let v = my_vec!(1, 2, 3);
    println!("{:?}", v);
    println!("{}", Test::hello());
}
