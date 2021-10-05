# Multi Key, Value Database

This is a learning projects.
This project is to have the Key be broken up so that they can be grouped and multi values received.

This repo provides a binary as well a library.

## Binary Example

adding

```text
kv_db -add -k "1.1.1" -v "important"
kv_db -add -k "1.1.2" -v "value"
kv_db -add -k "1.2" -v "something"
```

retrieving

```text
kv-db -get -k "1.1"
1.1.1 important
1.1.2 value

kv_db -get -k 1
1.1.1 important
1.1.2 value
1.2 something
```

## Running from src

get help

```text
cargo run -- -h
```

## Supported Commands

```text
USAGE:
    kv_db.exe [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help           Print help information
    -i, --interactive    Interactive Database mode
    -V, --version        Print version information

OPTIONS:
    -f, --file <db-file>...    Sets the db file for database
    -l, --log <log>            Sets the level of logging to output, default is off [possible values: error, warn, info, debug, trace]

SUBCOMMANDS:
    add       Add new key value to database
    get       Get value(s) from the database
    help      Print this message or the help of the given subcommand(s)
    print     Print Database to standard out
    remove    Remove value from the database
```
