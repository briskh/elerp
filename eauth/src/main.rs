use libelp::Configuration;

#[allow(dead_code)]
#[derive(Configuration)]
struct Test {
    #[config(default = "localhost", note = "redis host")]
    host: String,
    #[config(default = 5432, note = "redis port")]
    port: u16,
}

#[allow(dead_code)]
#[derive(Configuration, Debug)]
struct Redis {
    #[config(default = "localhost", note = "test host")]
    host: String,
    #[config(default = 5432, note = "test port")]
    port: u16,
}

#[derive(Configuration, Debug)]
struct Database {
    #[config(default = "localhost", note = "db host")]
    host: String,
    #[config(default = 5432, note = "db port")]
    port: u16,
}

#[derive(Configuration, Debug)]
struct Config {
    #[config(default = true, note = "enable feature")]
    feature: bool,
    database: Database,
    redis: Redis,
}

fn main() {
    // 测试默认值
    let config = Config::new();
    println!("{}", config.to_toml());
}

#[test]
fn test_config() {
    assert_eq!(main(), ());
}
