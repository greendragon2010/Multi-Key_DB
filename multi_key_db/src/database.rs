use crate::error::DBError;
use crate::error::KeyError;
use crate::key::Key;
use crate::node::NodeType;

use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use std::collections::BTreeMap;

use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use log::{debug, error, trace};

#[cfg(any(feature = "cli-features"))]
use cli_table::{
    format::{Align, Justify, Padding},
    print_stdout, Cell, Table,
};

type Result<T> = std::result::Result<T, DBError>;
const SPLIT_SETTING: &str = "split:";

/// Database for MultiKey DB
#[derive(Debug)]
pub struct Database<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr,
    V: Default + Debug + FromStr,
{
    // Structure
    // Key -> NodeType::Value -> Holds Value
    // Key -> NodeType::Parent
    //                   -> BTree Of Children same as this
    pub map: BTreeMap<Key<K>, NodeType<K, V>>,
    divider: char,
}

impl<K, V> Default for Database<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr + ToString,
    V: Default + Debug + FromStr + ToString,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Database<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr + ToString,
    V: Default + Debug + FromStr + ToString,
{
    pub fn new() -> Database<K, V> {
        Database {
            map: BTreeMap::new(),
            divider: '.', //default divider
        }
    }

    /// Creates a in memory database.
    /// Fill data from the file provided
    pub fn new_from_file<F>(file: &mut F) -> Result<Database<K, V>>
    where
        F: Read,
    {
        let mut database = Database::new();
        {
            let mut settings_read = false;
            let buff_reader = BufReader::new(file);
            for line in BufRead::lines(buff_reader) {
                let mut line = line?;
                if line.is_empty() || line.as_str().starts_with('#') {
                    continue;
                }
                if !settings_read && line.contains(SPLIT_SETTING) {
                    if line.len() == 7 {
                        database.divider = line.pop().unwrap();
                        settings_read = true;
                        continue;
                    } else {
                        return Err(DBError::CorruptDBFile);
                    }
                }
                let (key, value) = line.split_once("\t").ok_or(DBError::CorruptDBFile)?;
                match V::from_str(value) {
                    Ok(parsed_value) => {
                        database.insert(Key::new_from_str(key, database.divider)?, parsed_value)?
                    }
                    Err(_) => {
                        error!("Parse error, value of V: {0:#?}", value);
                        return Err(KeyError::ParseError.into());
                    }
                }
            }
        }

        trace!("Map: {0:#?}", database.map);

        debug!("DB Creation from file.");
        Ok(database)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, key: Key<K>, value: V) -> Result<()> {
        trace!("Insert, Key: {0:#?} Value: {1:#?}", key, value);

        if key.is_multi_key() {
            if let Some(node) = self.map.get_mut(&key.root()?) {
                match node {
                    NodeType::Parent(parent) => parent.insert(key.remove_root()?, value)?,
                    NodeType::Value(_) => return Err(DBError::MultiKeyExtendValueKey),
                }
            } else {
                let mut parent = Database::new();
                parent.insert(key.remove_root()?, value)?;
                self.map.insert(key.root()?, NodeType::Parent(parent));
            }
        } else if let Some(node) = self.map.get_mut(&key.root()?) {
            match node {
                NodeType::Parent(_) => return Err(DBError::InsertValueToDirectory),
                NodeType::Value(_) => {
                    self.map.insert(key, NodeType::Value(value));
                }
            }
        } else {
            self.map.insert(key, NodeType::Value(value));
        }

        trace!("DB: {0:#?}", self);
        Ok(())
    }

    pub fn get(&self, key: &Key<K>) -> Option<&V> {
        if let Some(value) = self.map.get(&key.root().unwrap()) {
            match value {
                NodeType::Parent(parent) => match key.remove_root() {
                    Ok(value) => parent.get(&value),
                    Err(_) => None,
                },
                NodeType::Value(value) => Some(value),
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &Key<K>) -> Option<V> {
        if key.is_multi_key() {
            if let Some(NodeType::Parent(parent)) = self.map.get_mut(&key.root().unwrap()) {
                return parent.remove(&key.remove_root().unwrap());
            }
        } else {
            let result = self.map.get(key);
            if result.is_some() && result.unwrap().is_value() {
                return self.map.remove(key).unwrap().get_value();
            }
        }
        None
    }

    pub fn get_values(&self, key: &Key<K>) -> Vec<(Key<K>, &V)> {
        if key.is_multi_key() {
            if let Some(NodeType::Parent(parent)) = self.map.get(&key.root().unwrap()) {
                let temps = parent.get_values(&key.remove_root().unwrap());
                let mut result = Vec::new();
                for (temp_key, value) in temps {
                    let mut root = key.root().unwrap();
                    root.add(&temp_key);
                    result.push((root, value));
                }
                return result;
            }
        } else if let Some(value) = self.map.get(key) {
            match value {
                NodeType::Parent(parent) => {
                    let temp_keys = parent.generate_key_value_pairs();
                    let mut result = Vec::new();
                    for (temp_key, value) in &temp_keys {
                        let mut key_clone = key.clone();
                        key_clone.add(temp_key);
                        result.push((key_clone, *value));
                    }
                    return result;
                }
                NodeType::Value(value) => return vec![(key.clone(), value)],
            }
        }
        Vec::new()
    }

    pub fn flush<F>(&mut self, file: &mut F) -> Result<()>
    where
        F: Write,
    {
        let mut contents = String::new();
        if self.divider != Database::<K, V>::default().divider {
            contents.push_str(SPLIT_SETTING);
            contents.push(self.divider);
            contents.push('\n');
        }
        let key_value_pairs = self.generate_key_value_pairs();
        for (key, value) in key_value_pairs {
            contents.push_str(&key.to_string(self.divider));
            contents.push('\t');
            contents.push_str(&value.to_string());
            contents.push('\n');
        }

        let mut writer = BufWriter::new(file);
        writer.write_all(contents.as_bytes())?;
        writer.flush()?;

        debug!("Flushed");
        Ok(())
    }

    #[cfg(any(feature = "cli-features"))]
    pub fn get_values_cli(&self, key: &Key<K>) -> Result<()> {
        let mut contents = Vec::new();
        let key_value_pairs = self.get_values(key);
        for (key, value) in key_value_pairs {
            contents.push(vec![
                key.to_string(self.divider)
                    .cell()
                    .align(Align::Center)
                    .padding(Padding::builder().right(1).build()),
                value.to_string().cell().align(Align::Center),
            ])
        }

        let table = contents.table().title(vec![
            "Key".cell().justify(Justify::Center).align(Align::Center),
            "Value".cell().justify(Justify::Center).align(Align::Center),
        ]);
        Ok(print_stdout(table)?)
    }

    #[cfg(any(feature = "cli-features"))]
    pub fn print_cli(&mut self) -> Result<()> {
        let mut contents = Vec::new();
        let key_value_pairs = self.generate_key_value_pairs();
        for (key, value) in key_value_pairs {
            contents.push(vec![
                key.to_string(self.divider)
                    .cell()
                    .align(Align::Center)
                    .padding(Padding::builder().right(1).build()),
                value.to_string().cell().align(Align::Center),
            ])
        }

        let table = contents.table().title(vec![
            "Key".cell().justify(Justify::Center).align(Align::Center),
            "Value".cell().justify(Justify::Center).align(Align::Center),
        ]);
        Ok(print_stdout(table)?)
    }

    pub fn generate_key_value_pairs(&self) -> Vec<(Key<K>, &V)> {
        let mut result = Vec::new();
        for (single_key, value) in &self.map {
            let key = Key::new_from_key(single_key);
            match value {
                NodeType::Parent(parent) => {
                    let temp_keys = parent.generate_key_value_pairs();
                    for (temp_key, value) in &temp_keys {
                        let mut key_clone = key.clone();
                        key_clone.add(temp_key);
                        result.push((key_clone, *value));
                    }
                }
                NodeType::Value(node_value) => {
                    result.push((key, node_value));
                }
            }
        }
        trace!("Result: {0:#?}", result);
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::database::*;
    use std::io::Cursor;

    /// This Creates a test struct that errors when Read::read(ReadError) is called.
    #[derive(Default)]
    struct ReadError;
    impl Read for ReadError {
        fn read(&mut self, _: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
            Err(std::io::ErrorKind::Other.into())
        }
    }
    /// Tests Database::new_in_memory()
    /// Creates a in memory DB
    #[test]
    fn database_default() {
        let db = Database::<usize, usize>::new();
        assert_eq!(0, db.map.len());
    }

    #[test]
    fn database_new_from_file_successful() -> Result<()> {
        let mut vector = "1.2.3.4.key\tvalue5".as_bytes().to_vec();
        let mut file = Cursor::new(&mut vector);

        let result = Database::<String, String>::new_from_file(&mut file);

        assert!(result.is_ok());
        let db = result.unwrap();
        assert_eq!(1, db.map.len());

        let one = db.map.get(&Key::new_from_str("1", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), one);
        let db_one = one.get_parent_ref().unwrap();
        assert_eq!(1, db_one.map.len());

        let two = db_one.map.get(&Key::new_from_str("2", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), two);
        let db_two = two.get_parent_ref().unwrap();
        assert_eq!(1, db_two.map.len());

        let three = db_two.map.get(&Key::new_from_str("3", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), three);
        let db_three = three.get_parent_ref().unwrap();
        assert_eq!(1, db_three.map.len());

        let four = db_three.map.get(&Key::new_from_str("4", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), four);
        let db_four = four.get_parent_ref().unwrap();
        assert_eq!(1, db_four.map.len());

        let key = db_four.map.get(&Key::new_from_str("key", '.')?).unwrap();
        assert_eq!(&NodeType::Value("".into()), key);
        let value = key.get_value_ref().unwrap();
        assert_eq!("value5", value);

        Ok(())
    }

    #[test]
    fn database_new_from_file_read_error() {
        let mut reader = ReadError::default();

        let result = Database::<usize, usize>::new_from_file(&mut reader);
        assert!(result.is_err());
        assert_eq!(
            DBError::IOError(std::io::ErrorKind::Other.into()),
            result.err().unwrap()
        );
    }
    #[test]
    fn database_new_from_file_corrupt_db() {
        let mut vector = "CorruptDB".as_bytes().to_vec();
        let mut file = Cursor::new(&mut vector);

        let result = Database::<usize, usize>::new_from_file(&mut file);
        assert!(result.is_err());
        assert_eq!(DBError::CorruptDBFile, result.err().unwrap());
    }

    #[test]
    fn database_insert() -> Result<()> {
        let mut db = Database::<usize, String>::new();
        let key = Key::<usize>::new_from_str("1.2.3.4.5", '.')?;

        let result = db.insert(key, "value-string".into());

        assert_eq!((), result.unwrap());

        assert_eq!(1, db.map.len());

        let one = db.map.get(&Key::new_from_str("1", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), one);
        let db_one = one.get_parent_ref().unwrap();
        assert_eq!(1, db_one.map.len());

        let two = db_one.map.get(&Key::new_from_str("2", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), two);
        let db_two = two.get_parent_ref().unwrap();
        assert_eq!(1, db_two.map.len());

        let three = db_two.map.get(&Key::new_from_str("3", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), three);
        let db_three = three.get_parent_ref().unwrap();
        assert_eq!(1, db_three.map.len());

        let four = db_three.map.get(&Key::new_from_str("4", '.')?).unwrap();
        assert_eq!(&NodeType::Parent(Database::new()), four);
        let db_four = four.get_parent_ref().unwrap();
        assert_eq!(1, db_four.map.len());

        let five = db_four.map.get(&Key::new_from_str("5", '.')?).unwrap();
        assert_eq!(&NodeType::Value("".into()), five);
        let value = five.get_value_ref().unwrap();
        assert_eq!("value-string", value);

        Ok(())
    }

    #[test]
    fn database_insert_root_insert_repeat() -> Result<()> {
        let mut db = Database::<usize, String>::new();
        let key = Key::new_from_str("1.2.3.4.5", '.')?;
        let root = Key::new_from_str("1.2.3.4", '.')?;

        let result = db.insert(key, "leaf".into());

        assert_eq!((), result.unwrap());
        assert_eq!(
            "leaf",
            db.get(&Key::new_from_str("1.2.3.4.5", '.')?).unwrap()
        );

        let root_result = db.insert(root, "root".into());

        assert!(root_result.is_err());
        assert_eq!(DBError::InsertValueToDirectory, root_result.err().unwrap());

        Ok(())
    }

    #[test]
    fn database_insert_key_value_key() -> Result<()> {
        let mut db = Database::<usize, String>::new();
        let key = Key::new_from_str("1.2.3.4.5", '.')?;
        let root = Key::new_from_str("1.2.3.4", '.')?;

        let result = db.insert(root, "leaf".into());

        assert_eq!((), result.unwrap());
        assert_eq!("leaf", db.get(&Key::new_from_str("1.2.3.4", '.')?).unwrap());

        let key_result = db.insert(key, "root".into());

        assert!(key_result.is_err());
        assert_eq!(DBError::MultiKeyExtendValueKey, key_result.err().unwrap());

        Ok(())
    }
}
