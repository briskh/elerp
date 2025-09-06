use libelp::Configuration;

#[allow(dead_code)]
#[derive(Configuration)]
struct Test {
    #[default = "8080"]
    #[note = "The port number of the server"]
    age: u16,
}

fn main() {
    println!("{}", Test::hello());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        assert_eq!(main(), ());
    }
}
