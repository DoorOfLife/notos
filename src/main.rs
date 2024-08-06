use toml::Table;
use std::fs::read_to_string;
use std::process::exit;
use clap::{Parser};

const OK: i32 = 0;
const VALIDATION_FAIL: i32 = 1;
const ROOT_DIR_DEFAULT: &str = ".notos";
const CONFIG_FILE_NAME: &str = "config";
const NOTES_DIR_DEFAULT: &str = "notes";
const LOG_FILE_DEFAULT: &str = ".log";

const CONFIG_KEY_NOTES_DIR: &str = "notes_dir";
const CONFIG_KEY_LOG_FILE: &str = "log_file";
const CONFIG_KEY_LOG_ENABLED: &str = "log_enabled";


#[derive(Parser)] // requires `derive` feature
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long = "destroy", help = "Destroy (delete) a topic file. This can not be undone.")]
    destroy: bool,

    #[arg(short = 'd', long = "delete-line", help = "Delete line, starting from 0")]
    delete_line: bool,

    #[arg(short = 'e', long = "edit", help = "Open the topic file in the default editor")]
    edit: bool,

    #[arg(short = 'a', long = "dump-all", help = "Prints all data from all topics")]
    dump_all: bool,

    topic: Option<String>,

    value: Vec<String>,
}

impl Cli {
    pub fn validate(&self) -> Option<&str> {
        if self.destroy {
            if self.delete_line {
                return Some("You can't both destroy a topic and also delete a line from it");
            } else if !self.value.is_empty() {
                return Some("If you wish to destroy a topic then do not add a note");
            } else if self.edit {
                return Some("You can't use edit and destroy in the same call, choose one");
            } else if self.dump_all {
                return Some("Destroying a topic and dumping all data are mutually exclusive");
            }
        } else if self.delete_line {
            if self.topic.is_none() {
                return Some("You need to specify a topic from which to delete a line");
            }
            if self.value.is_empty() {
                return Some("You need to specify a line to delete");
            }
            if self.dump_all {
                return Some("Either delete line or dump all data, not both");
            }
        } else if self.edit {
            if self.topic.is_none() {
                return Some("You need to specify a topic to edit");
            } else if self.dump_all {
                return Some("You must choose either to edit or dump all data");
            }
        } else if self.dump_all {
            if self.topic.is_some() || !self.value.is_empty() {
                return Some("If you wish to dump all data then do not provide any further arguments");
            }
        }
        return None;
    }
}

struct Config {
    notes_dir: String,
    log_file: String,
    log_enabled: bool,
}

fn fetch_config() -> Config {
    // Check if config file exists
    let home_dir = dirs::home_dir().expect("Could not find any home directory using dirs");
    let home_dir_str = home_dir.to_str().expect("Failed to convert dir to string");
    let config_file_path = home_dir.join(CONFIG_FILE_NAME);

    let mut config = Config {
        notes_dir: home_dir_str.to_owned() + "/" + ROOT_DIR_DEFAULT + "/" + NOTES_DIR_DEFAULT,
        log_file: home_dir_str.to_owned() + "/" + ROOT_DIR_DEFAULT + "/" + LOG_FILE_DEFAULT,
        log_enabled: true,
    };

    if config_file_path.exists() {
        // Load and read the file content
        let config_str = read_to_string(&config_file_path);
        match config_str {
            Ok(str) => {
                let parsed = str.parse::<Table>().expect("Failed to parse config file");
                let notes_dir = parsed.get(CONFIG_KEY_NOTES_DIR);
                let log_file = parsed.get(CONFIG_KEY_LOG_FILE);
                let log_enabled = parsed.get(CONFIG_KEY_LOG_FILE);

                if let Some(value) = notes_dir {
                    config.notes_dir = value.as_str().unwrap().to_owned();
                }

                if let Some(value) = log_file {
                    config.log_file = value.as_str().unwrap().to_owned();
                }

                if let Some(value) = log_enabled {
                    config.log_enabled = value.as_bool().unwrap();
                }

            }
            _ => {}
        }
    } 
    return config;
}

fn main() {
    let config = fetch_config();
    let args = Cli::parse();
    let error_msg = args.validate();
    if error_msg.is_some() {
        println!("{}", error_msg.unwrap());
        exit(VALIDATION_FAIL);
    }
    println!("Destroy? {:?}", args.destroy);
    println!("Delete_line? {:?}", args.delete_line);
    println!("Topic? {:?}", args.topic);
    println!("Value? {:?}", args.value);
    println!("Edit? {:?}", args.edit);
    println!("Dump all? {:?}", args.dump_all);
    println!("notes dir: {:?}", config.notes_dir);
    println!("log file: {:?}", config.log_file);
    println!("log enabled?: {:?}", config.log_enabled);


    // No args, print list of topics, return
    // Only topic arg, print it, return
    // Topic and value arg -> add to topic -> fall-thru:
    // edit flag -> open file in $EDITOR, return
    // destroy flag -> destroy topic
    // delete line flag -> delete line in topic
    // dump-all flag

    exit(OK);
}