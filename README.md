
So, parsing legacy messages with ssb-legacy-msg is ok.

doing a high performance flume codec parser doesn't need all the overhead of the validation stuff.
  - I'm assuming that this will be slower, where actually alj might have done some clever shit to parse super fast.
  - I might be able to hand off the deser to another thread and pass back the object that needs converting to napi?

Things I could do today: 
  - keep investigating js performance and try and squeeze more out.
    - timebox: until lunch
    - can see that using ssb-legacy-msg is a promising path, (what to do with content is the big unknown) but I expect it to be slow. All I need is a fast parse and throw into an object.
    - How do I use the vanilla json parser?
      - Can I embed a serde_json::Value inside another struct? 
  - finish exposing the bindings and call it done.
    - clmr
    - code docs
    - serialization for json and clmr
    - finishing touches on bindings still to do:
      - prebuild file format output and tar needs to be added to the cross scripts.

Might be able to build an object setting values by using a serde_json Value

