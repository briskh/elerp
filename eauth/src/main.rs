use libelp::ConfigStruct;

#[allow(dead_code)]
#[derive(ConfigStruct)]
struct Redis {
    #[config(default = "localhost", note = "redis host")]
    host: String,
    #[config(default = 5432, note = "redis port")]
    port: u16,
}

#[derive(ConfigStruct)]
struct Database {
    #[config(default = "localhost", note = "db host")]
    host: String,
    #[config(default = 5432, note = "db port")]
    port: u16,
    redis: Redis,
}

#[derive(ConfigStruct)]
struct Config {
    #[config(default = true, note = "enable feature")]
    feature: bool,
    database: Database,
}

fn main() {
    println!("Testing nested Config (two levels)");

    // 测试默认值
    let config = Config::new();
    println!(
        "Default config: feature={}, db.host={}, db.port={}",
        config.feature, config.database.host, config.database.port
    );

    // 测试to_toml方法
    let toml_output = config.to_toml();
    println!("TOML output:\n{}", toml_output);

    // 测试from_toml方法
    let toml_input = r#"
feature = false

[database]
host = "db.example"
port = 3306
"#;

    match Config::from_toml(toml_input) {
        Ok(parsed_config) => {
            println!(
                "Parsed config: feature={}, db.host={}, db.port={}",
                parsed_config.feature, parsed_config.database.host, parsed_config.database.port
            );
        }
        Err(e) => {
            println!("Error parsing TOML: {}", e);
        }
    }
}
