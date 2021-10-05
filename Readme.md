# Multi Key, Value Database

This is a learning projects.
This project is to have the Key be broken up so that they can be grouped and multi values received.

This provides a binary as well a library.

## Binary Example

adding
''kv_db -add -k "1.1.1" -v "important" ''
''kv_db -add -k "1.1.2" -v "value" ''
''kv_db -add -k "1.2" -v "something" ''

retrieving
''kv-db -get -k "1.1"
1.1.1 important
1.1.2 value''
''kv_db -get -k 1."
1.1.1 important
1.1.2 value
1.2 something''

## Running from src

cargo run ---bin kv_db -- binary commands
