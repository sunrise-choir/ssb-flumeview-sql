#include <node_api.h>

#include <assert.h>
#include <stdio.h>

extern "C" {
  extern napi_value parse_json_with_constructor(napi_env, napi_callback_info);
  extern napi_value parse_cbor_with_constructor(napi_env, napi_callback_info);

  extern napi_value parse_json(napi_env, napi_callback_info);
  extern napi_value parse_cbor(napi_env, napi_callback_info);

  extern napi_value to_json(napi_env, napi_callback_info);
  extern napi_value to_cbor(napi_env, napi_callback_info);
}

#define DECLARE_NAPI_METHOD(name, func)                          \
  { name, 0, func, 0, 0, 0, napi_default, 0 }

napi_value Init(napi_env env, napi_value exports) {
  napi_status status;
  napi_property_descriptor addDescriptor[] = {
    DECLARE_NAPI_METHOD("parseJson", parse_json),
    DECLARE_NAPI_METHOD("parseJsonWithConstructor", parse_json_with_constructor),
    DECLARE_NAPI_METHOD("parseCborWithConstructor", parse_cbor_with_constructor),
    DECLARE_NAPI_METHOD("parseCbor", parse_cbor),
    DECLARE_NAPI_METHOD("toCbor", to_cbor),
    DECLARE_NAPI_METHOD("toJson", to_json)
  };
  status = napi_define_properties(env, exports, 6 , addDescriptor);
  assert(status == napi_ok);
  return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
