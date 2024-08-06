use clap::Parser;
use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, LevelPadding, SimpleLogger, WriteLogger};
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::{fs, io};
use toml::Table;

const OK: i32 = 0;
const VALIDATION_FAIL: i32 = 1;
const TARGET_NOT_FOUND: i32 = 2;
const ROOT_DIR_DEFAULT: &str = ".notos";
const CONFIG_FILE_NAME: &str = "config.toml";
const NOTES_DIR_DEFAULT: &str = "notes";
const LOG_FILE_DEFAULT: &str = "notos.log";
const CONFIG_KEY_NOTES_DIR: &str = "notes_dir";
const CONFIG_KEY_LOG_FILE: &str = "log_file";
const CONFIG_KEY_LOG_ENABLED: &str = "log_enabled";
const FILE_EXTENSION_TOPICS: &str = ".txt";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(
        long = "destroy",
        help = "Destroy (delete) a topic file. This can not be undone."
    )]
    destroy: bool,

    #[arg(
        short = 'd',
        long = "delete-line",
        help = "Delete line, starting from 0"
    )]
    delete_line: bool,

    #[arg(
        short = 'e',
        long = "edit",
        help = "Open the topic file in the default editor"
    )]
    edit: bool,

    #[arg(
        short = 'a',
        long = "dump-all",
        help = "Prints all data from all topics"
    )]
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
                return Some(
                    "If you wish to dump all data then do not provide any further arguments",
                );
            }
        }
        return None;
    }
}

struct Config {
    notes_dir: PathBuf,
    log_file: PathBuf,
    log_level: String,
    log_enabled: bool,
}

fn fetch_config() -> Config {
    // Check if config file exists
    let home_dir = dirs::home_dir().expect("Could not find any home directory using dirs");
    let notos_root_dir = home_dir.join(ROOT_DIR_DEFAULT);
    let config_file_path = notos_root_dir.join(CONFIG_FILE_NAME);

    let mut config = Config {
        notes_dir: notos_root_dir.join(NOTES_DIR_DEFAULT),
        log_file: notos_root_dir.join(LOG_FILE_DEFAULT),
        log_level: "debug".to_string(),
        log_enabled: true,
    };

    if !notos_root_dir.exists() {
        match fs::create_dir_all(&notos_root_dir) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
    } else if config_file_path.exists() {
        // Load and read the file content
        let config_str = read_to_string(&config_file_path);
        match config_str {
            Ok(str) => {
                let parsed = str.parse::<Table>().expect("Failed to parse config file");
                let notes_dir = parsed.get(CONFIG_KEY_NOTES_DIR);
                let log_file = parsed.get(CONFIG_KEY_LOG_FILE);
                let log_enabled = parsed.get(CONFIG_KEY_LOG_FILE);

                if let Some(value) = notes_dir {
                    config.notes_dir = PathBuf::from(value.as_str().unwrap().to_owned());
                }

                if let Some(value) = log_file {
                    config.log_file = PathBuf::from(value.as_str().unwrap().to_owned());
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

fn bootstrap_dirs_and_logger(config: &Config) {
    if config.log_enabled {
        let log_file: File;
        if !config.log_file.exists() {
            if let Some(parent_dir) = config.log_file.parent() {
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir).expect("Failed to create log directory");
                }
            }
            log_file = File::create(&config.log_file).expect("Failed to create log file");
        } else {
            log_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&config.log_file)
                .expect("Failed to open log file");
        }

        let log_config = ConfigBuilder::new()
            .set_time_offset_to_local().expect("Failed to set local time for logger")
            .set_target_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_level_padding(LevelPadding::Right)
            .build();
        // Initialize the logger
        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::from_str(config.log_level.as_str()).expect("Unknown log level"),
            log_config,
            log_file,
        )])
        .expect("Failed to initialize logger");
    }
    if !config.notes_dir.exists() {
        fs::create_dir_all(&config.notes_dir).expect("Failed to create notes directory");
    }
}

fn destroy(file: &PathBuf) -> io::Result<()> {
    // Check if the path points to a file and not a directory
    if file.is_file() {
        // Attempt to remove the file
        match fs::remove_file(&file) {
            Ok(_) => {
                log::info!("File '{}' deleted successfully.", file.display());
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to delete file '{}': {}", file.display(), e);
                Err(e)
            }
        }
    } else {
        // Return an error if the path is not a file
        let err = Error::new(io::ErrorKind::InvalidInput, "Provided path is not a file");
        log::error!("Failed to delete file '{}': {}", file.display(), err);
        Err(err)
    }
}

fn delete_line(path: &PathBuf, line_num: usize) {}

fn add_to_file(path: &PathBuf, value: String) {}

fn open_file_in_editor(path: &PathBuf) {}

fn output_topic(path: &PathBuf) {}

fn dump_all_file_data_for_dir(path: &PathBuf) {}

fn print_all_topics_in_dir(path: &PathBuf) {}

fn main() {
    let config = fetch_config();
    bootstrap_dirs_and_logger(&config);
    let args = Cli::parse();
    let error_msg = args.validate();
    if error_msg.is_some() {
        log::error!("{}", error_msg.unwrap());
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

    if args.topic.is_some() {
        let target_topic = args.topic.unwrap();
        let target_path = config
            .notes_dir
            .join(&target_topic)
            .join(FILE_EXTENSION_TOPICS);

        if !args.value.is_empty() {
            // Both topic and value are present
            // delete line flag -> delete line in topic
            if args.delete_line {
                delete_line(
                    &target_path,
                    args.value[0]
                        .parse::<usize>()
                        .expect("Failed to parse line number"),
                );
            } else {
                // Add value to topic
                add_to_file(&target_path, args.value.join(" "));
            }
        } else {
            // Only topic is present
            // destroy flag -> destroy topic
            if args.destroy {
                if target_path.exists() {
                    destroy(&target_path).expect("Failed to destroy topic, see logs");
                } else {
                    log::warn!("Tried to destroy a non-existing topic {}", target_topic);
                    exit(TARGET_NOT_FOUND);
                }
            }
            // edit flag -> open file in $EDITOR, return
            if args.edit {
                open_file_in_editor(&target_path);
            } else {
                // Only topic arg, print it, return
                output_topic(&target_path);
            }
        }
    } else {
        // Neither topic nor value are present
        // dump-all flag
        if args.dump_all {
            dump_all_file_data_for_dir(&config.notes_dir);
        } else {
            // No args, print list of topics, return
            print_all_topics_in_dir(&config.notes_dir);
        }
    }

    log::trace!("This is a trace message."); // Won't be logged
    log::debug!("This is a debug message."); // Won't be logged
    log::info!("This is an info message."); // Should be logged
    log::warn!("This is a warning message."); // Should be logged
    log::error!("This is an error message."); // Should be logged
    exit(OK);
}
