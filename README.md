

## Notes

- how do I get the napi_env context into the serde stuff?
  => DeserializeSeed

- what should actually get returned by from_slice?  


- how would you serialize a napi value?

```
fn to_string(env: napi_env, callbackinfo: info){
  value: napi_value = ... // get arg 0

  val = match napi_get_valuetype(env, value) {
    napi_valuetype_napi_null => Value::Null(value) 
  };

  val.to_vec() //calls the serializer?
  ... //shove it into a buffer or a string as a napi_value
}
```

Some example json
```
{
  age: 3,
  alive: true,
  backpack: {
    something: false
  }
}
```
- the top level object is a napi_value with type object
  - create a new float64 napi_value for age. Set it to 3.
  - call set_property(env, obj, k, v)
- hypothesis: only objects actually return a napi_value. Err no?


