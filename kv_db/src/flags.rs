use crate::constants;
use clap::{self, App, AppSettings, Arg};

pub fn generate_app() -> App<'static> {
    App::new("Key Value Database")
        .add_version_author()
        .about(clap::crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(interactive_arg())
        .arg(db_file_arg())
        .arg(log_arg())
        .subcommand(add_subcommand())
        .subcommand(get_subcommand())
        .subcommand(remove_subcommand())
        .subcommand(print_subcommand())
}

fn interactive_arg() -> Arg<'static> {
    Arg::new(constants::INTERACTIVE)
        .short('i')
        .long("interactive")
        .about("Interactive Database mode")
        .takes_value(false)
}

fn db_file_arg() -> Arg<'static> {
    Arg::new(constants::FILE)
        .short('f')
        .long("file")
        .about("Sets the db file for database")
        .max_values(1)
        .takes_value(true)
}

fn log_arg() -> Arg<'static> {
    Arg::new(constants::LOG)
        .short('l')
        .long("log")
        .possible_values(&["error", "warn", "info", "debug", "trace"])
        .about("Sets the level of logging to output, default is off")
}

fn add_subcommand() -> App<'static> {
    App::new(constants::ADD)
        .about("Add new key value to database")
        .add_version_author()
        .arg(key_arg())
        .arg(value_arg())
}

fn get_subcommand() -> App<'static> {
    App::new(constants::GET)
        .about("Get value(s) from the database")
        .add_version_author()
        .arg(key_arg())
}

fn remove_subcommand() -> App<'static> {
    App::new(constants::REMOVE)
        .about("Remove value from the database")
        .add_version_author()
        .arg(key_arg())
}
fn print_subcommand() -> App<'static> {
    App::new(constants::PRINT)
        .about("Print Database to standard out")
        .add_version_author()
}

fn key_arg() -> Arg<'static> {
    Arg::new("key")
                        .short('k')
                        .long("key")
                        .about("Key value to use to find value to remove, Multi Key Structure This.Is.A.Multi.Key is delimited by period")
                        .takes_value(true)
                        .min_values(1)
                        .use_delimiter(true)
                        .value_delimiter('.')
                        .required(true)
}

fn value_arg() -> Arg<'static> {
    Arg::new("value")
        .short('v')
        .long("value")
        .about("Value to be added")
        .takes_value(true)
        .max_values(1)
        .use_delimiter(false)
        .required(true)
}

trait Extension {
    fn add_version_author(self) -> Self;
}

impl Extension for App<'_> {
    fn add_version_author(self) -> Self {
        self.version(clap::crate_version!())
            .author(clap::crate_authors!("\n"))
    }
}
