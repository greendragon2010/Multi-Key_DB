use std::fmt;
use std::io::Error;

#[derive(Debug)]
pub enum DBError {
    IOError(Error),
    KeyError(KeyError),
    CorruptDBFile,
    InsertValueToDirectory,
    MultiKeyExtendValueKey,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DBError::CorruptDBFile => writeln!(f, "Database is corrupted."),
            DBError::InsertValueToDirectory => {
                writeln!(f, "Trying to insert value, into a Key that's a directory")
            }
            DBError::MultiKeyExtendValueKey => {
                writeln!(
                    f,
                    "Trying to insert MultiKey trying to extend Key that has a value."
                )
            }
            DBError::KeyError(e) => writeln!(f, "{}", e),
            // The wrapped error contains additional information and is available
            // via the source() method.
            DBError::IOError(e) => writeln!(f, "{}", e),
        }
    }
}

impl std::error::Error for DBError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            DBError::CorruptDBFile => None,
            DBError::InsertValueToDirectory => None,
            DBError::MultiKeyExtendValueKey => None,
            DBError::KeyError(_) => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            DBError::IOError(ref e) => Some(e),
        }
    }
}

impl From<Error> for DBError {
    fn from(error: Error) -> DBError {
        DBError::IOError(error)
    }
}

impl From<KeyError> for DBError {
    fn from(error: KeyError) -> DBError {
        DBError::KeyError(error)
    }
}

impl PartialEq for DBError {
    fn eq(&self, other: &DBError) -> bool {
        match self {
            DBError::CorruptDBFile => matches!(other, DBError::CorruptDBFile),
            DBError::InsertValueToDirectory => matches!(other, DBError::InsertValueToDirectory),
            DBError::MultiKeyExtendValueKey => matches!(other, DBError::MultiKeyExtendValueKey),
            DBError::KeyError(_) => matches!(other, DBError::KeyError(_)),
            DBError::IOError(_) => matches!(other, DBError::IOError(_)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum KeyError {
    ParseError,
    NoKey,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            KeyError::ParseError => writeln!(f, "Could not parse input"),
            KeyError::NoKey => writeln!(f, "Input Key had a size of 0"),
        }
    }
}

impl std::error::Error for KeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
