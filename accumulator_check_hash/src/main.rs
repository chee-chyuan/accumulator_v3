use config::Config;

fn main() {
    let config_file_name = "config.json";

    let config = Config::new(&config_file_name);
    accumulator_check_hash::run(config);
    
}