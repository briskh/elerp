use libelp::Configuration;
#[allow(dead_code)]
#[derive(Configuration)]
struct Test {
    #[config(default = "8080", note = "The port number of the server")]
    port: u16,
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
