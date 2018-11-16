//! Data structures for storing and manipulating arbitrary legacy data.
#![allow(non_upper_case_globals)]

use std::fmt;
use std::ptr::{null, null_mut};

use serde::{
    de::{DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeMap, SerializeSeq, Serializer},
};

use napi::*;
use napi_sys::*;
use ssb_legacy_msg_data::LegacyF64;

pub struct NapiValue {
    pub env: napi_env,
    pub value: napi_value,
}

impl NapiValue {
    fn get_typeof(&self) -> napi_valuetype {
        get_typeof(self.env, self.value)
    }
}

impl Serialize for NapiValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.get_typeof() {
            napi_valuetype_napi_null => serializer.serialize_unit(),
            napi_valuetype_napi_boolean => {
                let b = wrap_unsafe_get(self.env, self.value, napi_get_value_bool);
                serializer.serialize_bool(b)
            }
            napi_valuetype_napi_number => {
                let n = wrap_unsafe_get(self.env, self.value, napi_get_value_double);
                serializer.serialize_f64(n)
            }
            napi_valuetype_napi_string => {
                let s = get_string(self.env, self.value).unwrap(); //Assume we're safe to unwrap here because we've already type checked the thing.
                serializer.serialize_str(&s)
            }
            napi_valuetype_napi_object => {
                let mut is_array = false;
                unsafe { napi_is_array(self.env, self.value, &mut is_array) };

                if is_array {
                    let array = NapiArray::from_existing(self.env, self.value);
                    let mut s = serializer.serialize_seq(Some(array.len()))?;
                    for value in array {
                        s.serialize_element(&NapiValue {
                            env: self.env,
                            value,
                        })?;
                    }
                    s.end()
                } else {
                    let object_map = get_object_map(self.env, self.value);
                    let mut m = serializer.serialize_map(Some(object_map.len()))?;
                    for (key, value) in object_map {
                        m.serialize_entry(
                            &key,
                            &NapiValue {
                                env: self.env,
                                value,
                            },
                        )?;
                    }
                    m.end()
                }
            }
            _ => serializer.serialize_unit(),
        }
    }
}

struct ValueVisitor {
    env: napi_env,
}

impl<'de> DeserializeSeed<'de> for NapiEnv {
    type Value = NapiValue;
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<NapiValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor { env: self.env })
    }
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = NapiValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid legacy ssb value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        let value = wrap_unsafe_create(self.env, v, napi_get_boolean);
        Ok(NapiValue {
            env: self.env,
            value,
        })
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        match LegacyF64::from_f64(v) {
            Some(_) => {
                let value = wrap_unsafe_create(self.env, v, napi_create_double);
                Ok(NapiValue {
                    env: self.env,
                    value,
                })
            }
            None => Err(E::custom("invalid float")),
        }
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        self.visit_f64(v as f64)
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        self.visit_f64(v as f64)
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        let value = create_string_utf8(self.env, &v);
        //let value = create_buffer_copy(self.env, v.as_bytes());
        Ok(NapiValue {
            env: self.env,
            value,
        })
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        let value = get_null_value(self.env);
        Ok(NapiValue {
            env: self.env,
            value,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut array = NapiArray::with_capacity(self.env, seq.size_hint().unwrap_or(0));

        while let Some(elem) = seq.next_element_seed(NapiEnv { env: self.env })? {
            array.push(elem.value);
        }

        Ok(NapiValue {
            env: self.env,
            value: array.array,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let object = create_object(self.env);
        let mut descriptors: Vec<napi_property_descriptor> = Vec::new();
            
        while let Some((key, val)) 
            = map.next_entry_seed(NapiEnv { env: self.env }, NapiEnv { env: self.env })? {
                let descriptor = napi_property_descriptor{
                    utf8name: null(), // key.as_ptr() as *const c_char,
                    name: key.value,
                    method: None,
                    getter: None,
                    setter: None,
                    value: val.value,
                    attributes: napi_property_attributes_napi_enumerable, 
                    data: null_mut() 
                };
                descriptors.push(descriptor);
            }

        let status = unsafe { napi_define_properties(self.env, object, descriptors.len(), descriptors.as_ptr() as * const napi_property_descriptor) };
        debug_assert!(status == napi_status_napi_ok);
        
        Ok(NapiValue {
            env: self.env,
            value: object,
        })
    }
}
