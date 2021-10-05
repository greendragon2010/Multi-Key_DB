use crate::database::Database;

use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Debug)]
pub enum NodeType<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr + Default,
    V: Default + Debug + Default + std::str::FromStr,
{
    Parent(Database<K, V>),
    Value(V),
}

impl<K, V> NodeType<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr + Default,
    V: Default + Debug + Default + std::str::FromStr,
{
    pub fn get_parent(self) -> Option<Database<K, V>> {
        match self {
            NodeType::Parent(parent) => Some(parent),
            NodeType::Value(_) => None,
        }
    }
    pub fn get_parent_ref(&self) -> Option<&Database<K, V>> {
        match self {
            NodeType::Parent(parent) => Some(parent),
            NodeType::Value(_) => None,
        }
    }

    pub fn is_parent(&self) -> bool {
        match self {
            NodeType::Parent(_) => true,
            NodeType::Value(_) => false,
        }
    }

    pub fn get_value_ref(&self) -> Option<&V> {
        match self {
            NodeType::Parent(_) => None,
            NodeType::Value(value) => Some(value),
        }
    }

    pub fn get_value(self) -> Option<V> {
        match self {
            NodeType::Parent(_) => None,
            NodeType::Value(value) => Some(value),
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            NodeType::Parent(_) => false,
            NodeType::Value(_) => true,
        }
    }
}

impl<K, V> PartialEq for NodeType<K, V>
where
    K: Debug + Eq + PartialEq + Hash + Ord + PartialOrd + Default + Clone + FromStr + Default,
    V: Default + Debug + Default + std::str::FromStr,
{
    fn eq(&self, other: &NodeType<K, V>) -> bool {
        match self {
            NodeType::Parent(_) => matches!(other, NodeType::Parent(_)),
            NodeType::Value(_) => matches!(other, NodeType::Value(_)),
        }
    }
}
