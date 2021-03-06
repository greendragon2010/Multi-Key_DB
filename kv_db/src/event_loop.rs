use crate::constants::{self};
use std::fs::OpenOptions;

use crate::Key;
use crate::KeyError;
use log::trace;
use multi_key_db::database::Database;
use multi_key_db::error::DBError;
use std::io::{self, Write};

pub fn event_loop(database: &mut Database<String, String>, db_file: &str) -> Result<(), DBError> {
    io::stdout().write_all(b"Support Commands:\n")?;
    io::stdout().write_all(b"Add    -k <key>... -v <value>...\n")?;
    io::stdout().write_all(b"Get    -k <key>...\n")?;
    io::stdout().write_all(b"Remove -k <key>...\n")?;
    io::stdout().write_all(b"Print\n")?;
    io::stdout().write_all(b"Save to Disk\n")?;
    io::stdout().write_all(b"Exit\n")?;
    loop {
        io::stdout().write_all(b"[kv db]")?;
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        if buffer.is_empty() {
            no_command_found();
            continue;
        }

        let split: Vec<&str> = buffer.split(' ').collect();
        if split.is_empty() {
            no_command_found();
            continue;
        }
        match split[0].trim() {
            constants::ADD => {
                if split.len() != 5 || split[1].trim() != "-k" || split[3] != "-v" {
                    eprintln!("Add requires -k <key> -v <value>");
                    continue;
                }
                let multi_key = create_key(split[2].trim())?;

                let mut value = split[4];
                while value.ends_with('\n') || value.ends_with('\r') {
                    value = value.trim_end_matches('\n');
                    value = value.trim_end_matches('\r');
                }

                add(database, multi_key, value.to_string())?
            }
            constants::GET => {
                if split.len() != 3 || split[1].trim() != "-k" {
                    eprintln!("Get requires -k <key> ");
                    continue;
                }
                let multi_key = create_key(split[2].trim())?;

                get(database, multi_key)?
            }
            constants::PRINT => flush_to_stdout(database)?,
            constants::REMOVE => {
                if split.len() != 3 || split[1].trim() != "-k" {
                    eprintln!("Remove requires -k <key> ");
                    continue;
                }
                let multi_key = create_key(split[2].trim())?;
                remove(database, multi_key)?
            }
            "exit" => {
                return Ok(());
            }
            constants::SAVE => {
                if let Err(error) = save_to_disk(database, db_file) {
                    eprintln!("Database writing to disk failure: {}", error);
                }
            }
            unsupported => {
                eprintln!("Unsupported command {}", unsupported);
            }
        }
    }
}

fn no_command_found() {
    eprintln!("No Command Entered");
}

fn create_key(key: &str) -> Result<Key<String>, KeyError> {
    Key::new_from_str(key, '.')
}

pub fn add(
    database: &mut Database<String, String>,
    key: Key<String>,
    value: String,
) -> Result<(), DBError> {
    if let Err(e) = database.insert(key, value) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let output = format!("Insert Error: {0}", e);
        handle.write_all(output.as_bytes())?;
        handle.flush()?;

        return Err(e);
    }

    Ok(())
}

pub fn flush_to_stdout(database: &mut Database<String, String>) -> Result<(), DBError> {
    database.print_cli()
}

pub fn get(database: &mut Database<String, String>, key: Key<String>) -> Result<(), DBError> {
    database.get_values_cli(&key)
}

pub fn remove(database: &mut Database<String, String>, key: Key<String>) -> Result<(), DBError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Some(value) = database.remove(&key) {
        let output = format!("Removed: {0}\n", value);
        handle.write_all(output.as_bytes())?;
    }
    handle.flush()?;
    Ok(())
}

pub fn save_to_disk(database: &mut Database<String, String>, db_file: &str) -> Result<(), DBError> {
    let mut writer = OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(db_file)?;
    trace!("Write Database file opened.");

    database.flush(&mut writer)
}
