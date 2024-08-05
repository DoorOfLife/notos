use std::process::exit;
use clap::{Parser};


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
        }
        return None;
    }

}

fn main() {
    let args = Cli::parse();
    let error_msg = args.validate();
    if error_msg.is_some() {
        println!("{}", error_msg.unwrap());
        exit(1);
    }
    println!("Destroy? {:?}", args.destroy);
    println!("Delete_line? {:?}", args.delete_line);
    println!("Topic? {:?}", args.topic);
    println!("Value? {:?}", args.value);
    exit(0);
}