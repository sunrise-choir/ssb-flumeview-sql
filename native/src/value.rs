//! Data structures for storing and manipulating arbitrary legacy data.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, btree_map};
use std::fmt;

use indexmap::{IndexMap, map};
use serde::{
    ser::{Serialize, Serializer, SerializeSeq, SerializeMap},
    de::{Deserialize, Deserializer, Visitor, SeqAccess, MapAccess, Error},
};

use ssb_legacy_msg_data::{LegacyF64, legacy_length};
use napi_sys::*;

// The maximum capacity of entries to preallocate for arrays and objects. Even if malicious input
// claims to contain a much larger collection, only this much memory will be blindly allocated.
static MAX_ALLOC: usize = 2048;

/// Represents any valid ssb legacy message value, preserving the order of object entries.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Value {
    /// The [null](https://spec.scuttlebutt.nz/datamodel.html#null) value.
    Null,
    /// A [boolean](https://spec.scuttlebutt.nz/datamodel.html#booleans).
    Bool(bool),
    /// A [float](https://spec.scuttlebutt.nz/datamodel.html#floats).
    Float(LegacyF64),
    /// A [string](https://spec.scuttlebutt.nz/datamodel.html#strings).
    String(String),
    /// An [array](https://spec.scuttlebutt.nz/datamodel.html#arrays).
    Array(Vec<Value>),
    /// An [object](https://spec.scuttlebutt.nz/datamodel.html#objects).
    Object(RidiculousStringMap<Value>),
}

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::Float(f) => serializer.serialize_f64(f.into()),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Array(ref v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for inner in v {
                    s.serialize_element(inner)?;
                }
                s.end()
            },
            Value::Object(ref m) => {
                let mut s = serializer.serialize_map(Some(m.len()))?;
                for (key, value) in m {
                    s.serialize_entry(&key, value)?;
                }
                s.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid legacy ssb value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        match LegacyF64::from_f64(v) {
            Some(f) => Ok(Value::Float(f)),
            None => Err(E::custom("invalid float"))
        }
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        // use the size hint, but put a maximum to the allocation because we can't trust the input
        let mut v = Vec::with_capacity(std::cmp::min(seq.size_hint().unwrap_or(0), MAX_ALLOC));

        while let Some(inner) = seq.next_element()? {
            v.push(inner);
        }

        Ok(Value::Array(v))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
        // use the size hint, but put a maximum to the allocation because we can't trust the input
        let mut m = RidiculousStringMap::with_capacity(std::cmp::min(map.size_hint().unwrap_or(0),
                                                         MAX_ALLOC));

        while let Some((key, val)) = map.next_entry()? {
            if let Some(_) = m.insert(key, val) {
                return Err(A::Error::custom("map had duplicate key"));
            }
        }

        Ok(Value::Object(m))
    }
}

/// Represents any valid ssb legacy message value that can be used as the content of a message,
/// preserving the order of object entries.
///
/// On deserialization, this enforces that the value is an object with a correct `type` entry.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ContentValue(pub Value);

impl Serialize for ContentValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContentValue {
    fn deserialize<D>(deserializer: D) -> Result<ContentValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ContentValueVisitor::new())
    }
}

struct ContentValueVisitor(bool);

impl ContentValueVisitor {
    fn new() -> ContentValueVisitor {
        ContentValueVisitor(true)
    }
}

impl<'de> Visitor<'de> for ContentValueVisitor {
    type Value = ContentValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid legacy ssb content value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(ContentValue(Value::Bool(v)))
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        match LegacyF64::from_f64(v) {
            Some(f) => Ok(ContentValue(Value::Float(f))),
            None => Err(E::custom("invalid float"))
        }
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(ContentValue(Value::String(v)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(ContentValue(Value::Null))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        // use the size hint, but put a maximum to the allocation because we can't trust the input
        let mut v = Vec::with_capacity(std::cmp::min(seq.size_hint().unwrap_or(0), MAX_ALLOC));

        while let Some(inner) = seq.next_element()? {
            v.push(inner);
        }

        Ok(ContentValue(Value::Array(v)))
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
        // use the size hint, but put a maximum to the allocation because we can't trust the input
        let mut m = RidiculousStringMap::with_capacity(std::cmp::min(map.size_hint().unwrap_or(0),
                                                         MAX_ALLOC));

        while let Some((key, val)) = map.next_entry::<String, Value>()? {
            if self.0 && key == "type" {
                match val {
                    Value::String(ref type_str) => {
                        if check_type_value(type_str) {
                            self.0 = false;
                        } else {
                            return Err(A::Error::custom("content had invalid type"));
                        }
                    }
                    _ => return Err(A::Error::custom("content type must be a string"))
                }

            }

            if let Some(_) = m.insert(key, val) {
                return Err(A::Error::custom("map had duplicate key"));
            }
        }

        if self.0 {
            return Err(A::Error::custom("content had no `type` entry"));
        }

        Ok(ContentValue(Value::Object(m)))
    }
}

/// Check whether the given string is a valid `type` value of a content object.
fn check_type_value(s: &str) -> bool{
    let len = legacy_length(s);

    if len < 3 || len > 53 {
        false
    } else {
        true
    }
}

/// A map with string keys that sorts strings according to
/// [object entry order](https://spec.scuttlebutt.nz/datamodel.html#signing-encoding-objects),
/// using insertion order for non-numeric keys.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RidiculousStringMap<V> {
    // Keys that parse as natural numbers, sorted numerically.
    naturals: BTreeMap<GraphicolexicalString, V>,
    // The remaining keys, sorted in insertion order.
    others: IndexMap<String, V>,
}

impl<V> RidiculousStringMap<V> {
    /// Create a new map with capacity for `n` key-value pairs. (Does not
    /// allocate if `n` is zero.)
    ///
    /// This only preallocates capacity for non-numeric strings.
    pub fn with_capacity(capacity: usize) -> RidiculousStringMap<V> {
        RidiculousStringMap {
            naturals: BTreeMap::new(),
            others: IndexMap::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.naturals.len() + self.others.len()
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    pub fn insert(&mut self, key: String, val: V) -> Option<V> {
        if key == "0" {
            self.naturals.insert(GraphicolexicalString(key), val)
        } else {
            if is_nat_str(&key) {
                self.naturals.insert(GraphicolexicalString(key), val)
            } else {
                self.others.insert(key, val)
            }
        }
    }

    /// Gets an iterator over the entries of the map. It first yields all entries with
    /// [numeric](https://spec.scuttlebutt.nz/datamodel.html#signing-encoding-objects) keys
    /// in ascending order, and then the remaining entries in the same order in
    /// which they were inserted.
    pub fn iter(&self) -> Iter<V> {
        Iter { naturals: self.naturals.iter(), others: self.others.iter(), nats: true }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &str) -> Option<&V>
    {
        if is_nat_str(key) {
            self.naturals.get(key)
        } else {
            self.others.get(key)
        }
    }
}

fn is_nat_str(s: &str) -> bool {
    match s.as_bytes().split_first() {
        Some((0x31...0x39, tail)) => {
            if tail.iter().all(|byte| *byte >= 0x30 && *byte <= 0x39) {
                true
            } else {
                false
            }
        }
        _ => {
            false
        },
    }
}

impl<'a, V> IntoIterator for &'a RidiculousStringMap<V> {
    type Item = (&'a String, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Iter<'a, V> {
        self.iter()
    }
}

/// An iterator over the entries of a [`RidiculousStringMap`](RidiculousStringMap), first
/// yielding all entries with
/// [numeric](https://spec.scuttlebutt.nz/datamodel.html#signing-encoding-objects) keys
/// in ascending order, and then yielding the remaining entries in the same order in
/// which they were inserted into the map.
pub struct Iter<'a, V> {
    naturals: btree_map::Iter<'a, GraphicolexicalString, V>,
    others: map::Iter<'a, String, V>,
    nats: bool,
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (&'a String, &'a V);

    fn next(&mut self) -> Option<(&'a String, &'a V)> {
        if self.nats {
            match self.naturals.next() {
                None => {
                    self.nats = false;
                    self.next()
                }
                Some((key, val)) => Some((&key.0, val)),
            }
        } else {
            self.others.next()
        }
    }
}

// A wrapper around String, that compares by length first and uses lexicographical order as a
// tie-breaker.
#[derive(PartialEq, Eq, Clone, Hash)]
struct GraphicolexicalString(String);

impl PartialOrd for GraphicolexicalString {
    fn partial_cmp(&self, other: &GraphicolexicalString) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GraphicolexicalString {
    fn cmp(&self, other: &GraphicolexicalString) -> Ordering {
        match self.0.len().cmp(&other.0.len()) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.0.cmp(&other.0),
        }
    }
}

impl fmt::Debug for GraphicolexicalString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Borrow<str> for GraphicolexicalString {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl From<String> for GraphicolexicalString {
    fn from(s: String) -> Self {
        GraphicolexicalString(s)
    }
}

impl From<GraphicolexicalString> for String {
    fn from(s: GraphicolexicalString) -> Self {
        s.0
    }
}
