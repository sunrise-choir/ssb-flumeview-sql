#!/bin/bash
bindgen node_api.h -o bindings.rs --whitelist-function napi_.+ --whitelist-var napi_.+ --whitelist-type napi_.+
