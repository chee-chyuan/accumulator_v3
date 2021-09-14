use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub block_connection_string: String,
    pub master_accumulator_file_path: String,
    pub epoch_accumulator_file_path: String,
    pub starting_block_number: u32,
}

impl Config {
    pub fn new(file_name: &str) -> Config {
        let contents = fs::read_to_string(file_name).expect("Something went wrong reading the file");

        let config: Config = serde_json::from_str(&contents).expect("JSON was not well-formatted");

        config
    }
}