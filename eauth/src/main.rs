use libelp::ConfigStruct;

#[allow(dead_code)]
#[derive(ConfigStruct)]
struct Config {
    #[config(default = "localhost", note = "The host address")]
    host: String,
}

fn main() {
    println!("Testing Config macro with TOML functionality");

    // 测试默认值
    let config = Config::new();
    println!("Default config: host={}", config.host);

    // 测试to_toml方法
    let toml_output = config.to_toml();
    println!("TOML output:\n{}", toml_output);

    // 测试from_toml方法
    let toml_input = r#"
host = "example.com"
"#;

    match Config::from_toml(toml_input) {
        Ok(parsed_config) => {
            println!("Parsed config: host={}", parsed_config.host);
        }
        Err(e) => {
            println!("Error parsing TOML: {}", e);
        }
    }
}
