mod constants;
mod event_loop;
mod flags;

use crate::event_loop::save_to_disk;
use clap::ArgMatches;
use event_loop::{add, event_loop, flush_to_stdout, get, remove};
use log::{debug, error, trace, warn, LevelFilter};
use multi_key_db::{database::Database, error::DBError, error::KeyError, key::Key};
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::str::FromStr;

fn main() {
    let matches = flags::generate_app().get_matches();

    let log_level = matches
        .value_of(constants::LOG)
        .map(|m| LevelFilter::from_str(m).unwrap_or(LevelFilter::Off))
        .unwrap_or(LevelFilter::Off);

    set_logger(log_level);

    let mut db_file = constants::retrieve_db_file();
    if let Some(config) = matches.value_of(constants::FILE) {
        db_file = config.into();
    }
    debug!("Database File Location: {}", db_file);

    let db = create_db(&db_file);
    if let Err(error) = db {
        error!("Database creation error: {}", error);
        return;
    }
    let mut db = db.unwrap();

    if matches.is_present(constants::INTERACTIVE) {
        if let Err(error) = event_loop(&mut db, &db_file) {
            error!("Interactive error {}", error);
            return;
        };
    } else {
        match matches.subcommand() {
            Some((constants::ADD, add_command)) => {
                let multi_key = retrieve_key(add_command);
                match multi_key {
                    Err(e) => {
                        error!("Key creation error: {}", e);
                        return;
                    }
                    Ok(key) => {
                        let value = add_command.value_of("value").unwrap();

                        if let Err(e) = add(&mut db, key, value.to_string()) {
                            error!("Database Add Error: {}", e);
                            return;
                        }
                    }
                }
            }
            Some((constants::GET, get_command)) => {
                let multi_key = retrieve_key(get_command);
                match multi_key {
                    Err(e) => {
                        error!("Key creation error: {}", e);
                        return;
                    }
                    Ok(key) => {
                        if let Err(e) = get(&mut db, key) {
                            error!("Database Get Error: {}", e);
                            return;
                        }
                    }
                }
            }
            Some((constants::REMOVE, remove_command)) => match retrieve_key(remove_command) {
                Err(e) => {
                    error!("Key creation error: {}", e);
                    return;
                }
                Ok(key) => {
                    if let Err(e) = remove(&mut db, key) {
                        error!("Database Remove Error: {}", e);
                        return;
                    }
                }
            },
            Some((constants::PRINT, _)) => {
                if let Err(error) = flush_to_stdout(&mut db) {
                    error!("Database writing to standard out failure: {}", error);
                }
            }
            _ => (),
        }
    }

    if let Err(error) = save_to_disk(&mut db, &db_file) {
        error!("Database writing to disk failure: {}", error);
        return;
    }
}
fn retrieve_key(matches: &ArgMatches) -> Result<Key<String>, KeyError> {
    Key::new_from_vec(
        matches
            .values_of("key")
            .unwrap()
            .map(|s| s.into())
            .collect(),
    )
}

fn create_db(db_file: &str) -> Result<Database<String, String>, DBError> {
    let reader = OpenOptions::new().read(true).open(db_file);
    trace!("Read Database file opened.");

    match reader {
        Ok(mut reader) => Database::new_from_file(&mut reader),
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                warn!("File not found, empty database created.");
                Ok(Database::new())
            } else {
                Err(DBError::IOError(error))
            }
        }
    }
}

fn set_logger(log_level: LevelFilter) {
    env_logger::builder()
        .format_indent(Some(1))
        .format_timestamp(None)
        .filter_level(log_level)
        .init();
}
