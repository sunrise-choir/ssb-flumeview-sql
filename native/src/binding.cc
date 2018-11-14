#include <node_api.h>

#include <assert.h>
#include <stdio.h>

extern "C" {
  extern napi_value parse_legacy(napi_env, napi_callback_info);
  extern napi_value parse_cbor(napi_env, napi_callback_info);

  extern napi_value stringify_legacy(napi_env, napi_callback_info);
  extern napi_value encode_cbor(napi_env, napi_callback_info);
}

#define DECLARE_NAPI_METHOD(name, func)                          \
  { name, 0, func, 0, 0, 0, napi_default, 0 }

napi_value Init(napi_env env, napi_value exports) {
  napi_status status;
  napi_property_descriptor addDescriptor[] = {
    DECLARE_NAPI_METHOD("parseLegacy", parse_legacy),
    DECLARE_NAPI_METHOD("parseCbor", parse_cbor),
    DECLARE_NAPI_METHOD("encodeCbor", encode_cbor),
    DECLARE_NAPI_METHOD("stringifyLegacy", stringify_legacy)
  };
  status = napi_define_properties(env, exports, 4, addDescriptor);
  assert(status == napi_ok);
  return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
