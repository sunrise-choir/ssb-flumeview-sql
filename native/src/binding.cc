#include <node_api.h>

#include <assert.h>
#include <stdio.h>

extern "C" {
  extern napi_value define_view_class(napi_env);
}

#define DECLARE_NAPI_METHOD(name, func)                          \
  { name, 0, func, 0, 0, 0, napi_default, 0 }

//DECLARE_NAPI_METHOD("parseJson", parse_json),
//status = napi_define_properties(env, exports, sizeof(addDescriptor) / sizeof(addDescriptor[0]) , addDescriptor);

napi_value Init(napi_env env, napi_value exports) {
  return define_view_class(env);
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
