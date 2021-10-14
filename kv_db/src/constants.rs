use dirs_next::data_dir;
//Sub-commands
pub(crate) const ADD: &str = "add";
pub(crate) const GET: &str = "get";
pub(crate) const REMOVE: &str = "remove";
pub(crate) const PRINT: &str = "print";
pub(crate) const LOG: &str = "log";
pub(crate) const SAVE: &str = "save";
//commands
pub(crate) const INTERACTIVE: &str = "interactive";
pub(crate) const FILE: &str = "db-file";

pub fn retrieve_db_file() -> String {
    let db_file: String = ".kv.db".into();

    match data_dir() {
        Some(mut path) => {
            path.push("KV_DB");
            if std::fs::create_dir_all(&path).is_err() {
                return db_file;
            }
            path.push(&db_file);
            match path.to_str() {
                Some(path_str) => path_str.into(),
                None => db_file,
            }
        }
        None => db_file,
    }
}
