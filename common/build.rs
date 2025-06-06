use std::fs::{create_dir_all, read_to_string, write};
use std::path::Path;
use std::sync::LazyLock;

use heck::ToShoutySnakeCase;
use regex::Regex;

static CMD_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\sCmd\w*\s=\s\d+"#).unwrap());

const OUTPUT_DIR: &str = "include/proto/";
const CMD_ID_OUTPUT_FILE: &str = "./include/proto/cmd.rs";
const PROTO_FILE: &str = "StarRail.proto";

fn main() -> std::io::Result<()> {
    if !Path::new(OUTPUT_DIR).exists() {
        create_dir_all(OUTPUT_DIR)?;
    }

    if Path::new(PROTO_FILE).exists() {
        prost_build::Config::new()
            .out_dir(OUTPUT_DIR)
            .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
            .compile_protos(&[PROTO_FILE], &["."])
            .unwrap();

        let cmd_ids = parse_cmd_ids(PROTO_FILE)?;
        let cmd_ids = cmd_ids.join("\n");

        write(CMD_ID_OUTPUT_FILE, &cmd_ids)?;
        println!("cargo::rerun-if-changed={}", PROTO_FILE);

        Ok(())
    } else {
        panic!("{} is missing.", PROTO_FILE);
    }
}

fn parse_cmd_ids(file_path: &str) -> std::io::Result<Vec<String>> {
    let file_content = read_to_string(file_path)?;
    let mut result = Vec::with_capacity(2049);

    for cap in CMD_ID_REGEX.captures_iter(&file_content) {
        if let Some(cmd_line) = cap.get(0) {
            let cmd_line = cmd_line.as_str();
            let cmd_line = cmd_line.replace("\tCmd", "");
            let cmd_parts = cmd_line.split(" = ").collect::<Vec<&str>>();
            assert_eq!(cmd_parts.len(), 2);
            let cmd_name = cmd_parts[0].to_shouty_snake_case();
            let cmd_id = cmd_parts[1].parse::<u16>().unwrap();
            let cmd_const = format!("pub const {}: u16 = {};", cmd_name, cmd_id);
            result.push(cmd_const)
        }
    }

    Ok(result)
}
