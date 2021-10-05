use crate::error::KeyError;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

type Result<T> = std::result::Result<T, KeyError>;

#[derive(Debug, Default, Clone)]
pub struct Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    multi_key: Vec<K>,
    parent_depth: usize,
}

impl<K> Key<K>
where
    K: FromStr + Debug + Eq + PartialEq + Hash + Ord + PartialOrd + ToString,
{
    pub fn new_from_string(key: String, divider: char) -> Result<Key<K>> {
        Key::new_from_str(&key, divider)
    }

    pub fn new_from_str(key: &str, divider: char) -> Result<Key<K>> {
        let mut vec = Vec::new();
        let split_key = key.split(divider);
        for i in split_key {
            match K::from_str(i) {
                Ok(parsed) => vec.push(parsed),
                Err(_) => return Err(KeyError::ParseError),
            }
        }
        Key::new_from_vec(vec)
    }

    pub fn to_string(&self, divider: char) -> String {
        let mut result = String::new();

        for key in &self.multi_key {
            result.push_str(&key.to_string());
            result.push(divider);
        }
        result.pop();

        result
    }
}

impl<K> Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    pub fn new_from_vec(multi_key: Vec<K>) -> Result<Key<K>> {
        if multi_key.is_empty() {
            Err(KeyError::NoKey)
        } else {
            Ok(Key {
                multi_key,
                parent_depth: 0,
            })
        }
    }

    pub fn is_multi_key(&self) -> bool {
        self.multi_key.len() > 1
    }

    fn increment_parent(&mut self) {
        self.parent_depth += 1;
    }

    pub fn size(&self) -> usize {
        self.multi_key.len()
    }
}
impl<K> Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Clone,
{
    pub fn root(&self) -> Result<Key<K>> {
        let vec = vec![self.multi_key[0].clone()];
        Key::new_from_vec(vec)
    }

    pub fn remove_root(&self) -> Result<Key<K>> {
        let mut vec = Vec::new();
        for i in 1..self.multi_key.len() {
            vec.push(self.multi_key[i].clone());
        }

        Key::new_from_vec(vec)
    }
    pub fn parent(&self) -> Result<Key<K>> {
        let mut vec = Vec::new();
        for i in 0..self.multi_key.len() - 1 {
            vec.push(self.multi_key[i].clone());
        }

        Key::new_from_vec(vec)
    }

    pub fn add(&mut self, key: &Key<K>) {
        for value in &key.multi_key {
            self.multi_key.push(value.clone());
        }
    }

    pub fn new_from_key(other: &Key<K>) -> Key<K> {
        let mut key = Key {
            multi_key: Vec::new(),
            parent_depth: 0,
        };
        key.add(other);
        key
    }

    pub fn get_inner_key(&self, index: usize) -> Option<Key<K>> {
        if index >= self.multi_key.len() {
            None
        } else {
            Some(Key {
                multi_key: vec![self.multi_key[index].clone()],
                parent_depth: 0,
            })
        }
    }
}

impl<K> Iterator for Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Clone,
{
    type Item = Key<K>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.parent_depth >= self.multi_key.len() {
            self.parent_depth = 0;
            return None;
        }
        let mut result = Vec::with_capacity(self.parent_depth);
        for i in 0..=self.parent_depth {
            result.push(self.multi_key[i].clone());
        }
        self.increment_parent();
        match Key::new_from_vec(result) {
            Ok(key) => Some(key),
            Err(_) => None,
        }
    }
}

impl<K> Eq for Key<K> where K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd {}
impl<K> PartialEq for Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    fn eq(&self, other: &Key<K>) -> bool {
        self.multi_key.eq(&other.multi_key)
    }
}
impl<K> Hash for Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.multi_key.hash(hasher);
    }
}
impl<K> Ord for Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    fn cmp(&self, other: &Key<K>) -> std::cmp::Ordering {
        self.multi_key.cmp(&other.multi_key)
    }
}
impl<K> PartialOrd for Key<K>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd,
{
    fn partial_cmp(&self, other: &Key<K>) -> std::option::Option<std::cmp::Ordering> {
        self.multi_key.partial_cmp(&other.multi_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::KeyError;
    use crate::key::Key;

    #[test]
    fn test_new_empty_str() -> Result<(), KeyError> {
        let key: Key<String> = Key::new_from_str("", '.')?;

        assert_eq!(1, key.multi_key.len());

        Ok(())
    }

    #[test]
    fn test_new_empty_vec() {
        assert_eq!(Err(KeyError::NoKey), Key::<usize>::new_from_vec(vec![]));
    }

    #[test]
    fn key_iterator_str() -> Result<(), KeyError> {
        let mut key: Key<String> = Key::new_from_str("1.2.3", '.')?;
        assert_eq!(
            &Some(Key::<String>::new_from_str("1", '.')?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<String>::new_from_str("1.2", '.')?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<String>::new_from_str("1.2.3", '.')?),
            &mut key.next()
        );
        assert_eq!(&None, &mut key.next());
        assert_eq!(
            &Some(Key::<String>::new_from_str("1", '.')?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<String>::new_from_str("1.2", '.')?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<String>::new_from_str("1.2.3", '.')?),
            &mut key.next()
        );
        assert_eq!(&None, &mut key.next());

        Ok(())
    }
    #[test]
    fn key_iterator_int() -> Result<(), KeyError> {
        let mut key: Key<usize> = Key::new_from_vec(vec![1, 2, 3])?;
        assert_eq!(&Some(Key::<usize>::new_from_vec(vec![1])?), &mut key.next());
        assert_eq!(
            &Some(Key::<usize>::new_from_vec(vec![1, 2])?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<usize>::new_from_vec(vec![1, 2, 3])?),
            &mut key.next()
        );
        assert_eq!(&None, &mut key.next());
        assert_eq!(&Some(Key::<usize>::new_from_vec(vec![1])?), &mut key.next());
        assert_eq!(
            &Some(Key::<usize>::new_from_vec(vec![1, 2])?),
            &mut key.next()
        );
        assert_eq!(
            &Some(Key::<usize>::new_from_vec(vec![1, 2, 3])?),
            &mut key.next()
        );
        assert_eq!(&None, &mut key.next());

        Ok(())
    }
}
