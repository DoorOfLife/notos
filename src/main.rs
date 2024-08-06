use std::process::exit;
use clap::{Parser};

const OK: i32 = 0;
const VALIDATION_FAIL: i32 = 1;

#[derive(Parser)] // requires `derive` feature
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long = "destroy", help = "Destroy (delete) a topic file. This can not be undone.")]
    destroy: bool,

    #[arg(short = 'd', long = "delete-line", help = "Delete line, starting from 0")]
    delete_line: bool,

    #[arg(short = 'e', long = "edit", help = "Open the topic file in the default editor")]
    edit: bool,

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
            }
        } else if self.delete_line {
            if self.topic.is_none() {
                return Some("You need to specify a topic from which to delete a line");
            }
            if self.value.is_empty() {
                return Some("You need to specify a line to delete");
            }
        } else if self.edit {
            if self.topic.is_none() {
                return Some("You need to specify a topic to edit");
            }
        }
        return None;
    }

}

fn main() {
    // if .notos exists load config
    // if $NOTOS_NOTES exists use this path
    // otherwise use default path (/home/$user/.notos and equivalent on win)
    // if $NOTOS_LOG exists use this path
    // otherwise use default path (/home/$user/.notos.log ---||---)
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

    // No args, print list of topics, return
    // Only topic arg, print it, return
    // Topic and value arg -> add to topic -> fall-thru:
    // edit flag -> open file in $EDITOR, return
    // destroy flag -> destroy topic
    // delete line flag -> delete line in topic

    exit(OK);
}