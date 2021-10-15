use assert_cmd::prelude::*; // Add methods on commands
use multi_key_db::database::Database;
use multi_key_db::key::Key;
use predicates::prelude::*; // Used for writing assertions
use std::error::Error;
use std::fs::OpenOptions;
use std::process::Command;
use tempfile::tempdir; // Run programs

#[test]
fn no_args() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("kv_db")?;
    cmd.assert()
        .success()
        .stderr(predicate::str::contains(construct_help()));

    Ok(())
}

#[test]
fn help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("kv_db")?;
    cmd.arg("-h");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(construct_help()));

    Ok(())
}

#[test]
fn print() -> Result<(), Box<dyn Error>> {
    let mut database = Database::<String, String>::new();

    let key = Key::new_from_vec(vec!["work".into(), "team".into(), "git".into()]).unwrap();

    database.insert(key, "github.com/example-repo".into())?;

    {
        let dir = tempdir()?;
        let db_file = dir.path().join("kv.db");

        println!("Path: {:?}", db_file);
        let mut writer = OpenOptions::new()
            .read(false)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&db_file)?;

        database.flush(&mut writer)?;

        let mut cmd = Command::cargo_bin("kv_db")?;
        cmd.arg("-f");
        cmd.arg(db_file);
        cmd.arg("print");

        let output = predicate::str::contains("work.team.git")
            .and(predicate::str::contains("github.com/example-repo"));

        cmd.assert().success().stdout(output);
    }

    Ok(())
}

fn construct_help() -> String {
    let mut output = String::new();
    output.push_str("Key Value Database ");
    output.push_str(env!("CARGO_PKG_VERSION"));
    output.push_str("\n\n");
    output.push_str(env!("CARGO_PKG_AUTHORS"));
    output.push_str("\n\n");
    output.push_str(
        r"A Command Line Key Value Store Database

USAGE:
    kv_db.exe [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help           Print help information
    -i, --interactive    Interactive Database mode
    -V, --version        Print version information

OPTIONS:
    -f, --file <db-file>...    Sets the db file for database
    -l, --log <log>            Sets the level of logging to output, default is off [possible values:
                               error, warn, info, debug, trace]

SUBCOMMANDS:
    add       Add new key value to database
    get       Get value(s) from the database
    help      Print this message or the help of the given subcommand(s)
    print     Print Database to standard out
    remove    Remove value from the database",
    );
    output
}
