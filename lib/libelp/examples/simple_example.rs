use libelp::Configuration;

#[derive(Configuration, Debug)]
struct AppConfig {
    #[config(default = "localhost", note = "Server hostname")]
    host: String,
    #[config(default = 8080, note = "Server port")]
    port: u16,
}

fn main() {
    let cfg = AppConfig::new();
    // Show generated commented TOML
    let toml_str = cfg.to_toml();
    println!("{}", toml_str);
}


