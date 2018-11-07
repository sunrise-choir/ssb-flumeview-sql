

## Notes

- how do I get the napi_env context into the serde stuff?
  => DeserializeSeed

- what should actually get returned by from_slice?  


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


