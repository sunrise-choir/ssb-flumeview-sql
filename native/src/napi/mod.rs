use errors::*;
use napi_sys::*;
use std::debug_assert;
use std::ffi::{CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::collections::BTreeMap;
use std::any::TypeId;

pub fn wrap_unsafe_create<T>(env: napi_env, item: T, f: unsafe extern "C" fn(napi_env, T, *mut napi_value)->napi_status) -> napi_value{
    let mut result: napi_value = ptr::null_mut();
    let status = unsafe{f(env, item, &mut result)};
    debug_assert!(status == napi_status_napi_ok);
    result
}

pub fn wrap_unsafe_get<T: Default>(env: napi_env, value: napi_value, f: unsafe extern "C" fn(napi_env, napi_value, *mut T)->napi_status) -> T{
    let mut result: T = T::default();
    let status = unsafe{f(env, value, &mut result)};
    debug_assert!(status == napi_status_napi_ok);
    result
}

pub fn throw_error(env: napi_env, err: ErrorKind) {
    let status: napi_status;
    let msg = CString::new(err.description()).unwrap();
    unsafe {
        status = napi_throw_error(env, ptr::null(), msg.as_ptr() as *const c_char);
    }
    debug_assert!(status == napi_status_napi_ok)
}

pub fn create_error(env: napi_env, err: ErrorKind) -> napi_value {
    let status: napi_status;
    let mut result: napi_value = ptr::null_mut();
    let msg = create_string_utf8(env, err.description());

    unsafe {
        status = napi_create_error(env, ptr::null_mut(), msg, &mut result);
    }
    debug_assert!(status == napi_status_napi_ok);

    result
}
pub fn get_undefined_value(env: napi_env) -> napi_value {
    let mut undefined_value: napi_value = ptr::null_mut();
    let status: napi_status;
    unsafe {
        status = napi_get_undefined(env, &mut undefined_value);
    }
    debug_assert!(status == napi_status_napi_ok);

    undefined_value
}

pub fn get_null_value(env: napi_env) -> napi_value {
    let mut null_value: napi_value = ptr::null_mut();
    let status = unsafe {
        napi_get_null(env, &mut null_value)
    };
    debug_assert!(status == napi_status_napi_ok);

    null_value
}

pub fn get_arg(env: napi_env, info: napi_callback_info, arg_index: usize) -> napi_value {
    let status: napi_status;
    let mut num_args = arg_index + 1;
    let mut args: Vec<napi_value> = Vec::with_capacity(num_args);

    unsafe {
        status = napi_get_cb_info(
            env,
            info,
            &mut num_args,
            args.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        args.set_len(num_args);
    }

    debug_assert!(status == napi_status_napi_ok);

    args[arg_index]
}

pub fn check_is_buffer(env: napi_env, value: napi_value) -> bool {
    let status: napi_status;
    let mut result = false;
    unsafe { status = napi_is_buffer(env, value, &mut result) }
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn get_buffer_info(env: napi_env, buffer: napi_value) -> (*mut u8, usize) {
    let status: napi_status;
    let mut buff_size = 0;
    let mut p_buff: *mut c_void = ptr::null_mut();

    unsafe {
        status = napi_get_buffer_info(env, buffer, &mut p_buff, &mut buff_size);
    }
    debug_assert!(status == napi_status_napi_ok);

    (p_buff as *mut u8, buff_size)
}

pub fn create_buffer_copy(env: napi_env, slice: &[u8]) -> napi_value {
    let status: napi_status;
    let mut _p_buff: *mut c_void = ptr::null_mut();
    let mut buffer: napi_value = ptr::null_mut();

    unsafe {
        status = napi_create_buffer_copy(
            env,
            slice.len(),
            slice.as_ptr() as *const c_void,
            &mut _p_buff,
            &mut buffer,
        );
    }

    debug_assert!(status == napi_status_napi_ok);

    buffer
}

pub fn create_string_utf8(env: napi_env, string: &str) -> napi_value {
    let status: napi_status;
    let mut result: napi_value = ptr::null_mut();
    let p_str: *const std::os::raw::c_char = string.as_ptr() as *const c_char;

    unsafe {
        status = napi_create_string_utf8(env, p_str, string.len(), &mut result);
    }
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn get_string(env: napi_env, value: napi_value) -> Result<String> {
    let max_string_size = 100;

    let vec: Vec<u8> = Vec::with_capacity(max_string_size);
    let mut cstr = unsafe { CString::from_vec_unchecked(vec) };
    let p_str = cstr.into_raw();
    let mut length = 0;

    let status = unsafe {napi_get_value_string_utf8(env, value, p_str, max_string_size, &mut length)};
    if status == napi_status_napi_string_expected{
        bail!(ErrorKind::StringError)
    }

    debug_assert!(status == napi_status_napi_ok);

    cstr = unsafe{ CString::from_raw(p_str)};

    cstr.into_string()
        .or(Err(ErrorKind::StringError.into()))

}

pub fn create_buffer(env: napi_env, len: usize) -> napi_value {
    let status: napi_status;
    let mut _p_buff: *mut c_void = ptr::null_mut();
    let mut buffer: napi_value = ptr::null_mut();

    unsafe {
        status = napi_create_buffer(env, len, &mut _p_buff, &mut buffer);
    }
    debug_assert!(status == napi_status_napi_ok);

    buffer
}

pub fn create_reference(env: napi_env, value: napi_value) -> napi_ref {
    let status: napi_status;
    let mut reference: napi_ref = ptr::null_mut();

    unsafe {
        status = napi_create_reference(env, value, 1, &mut reference);
    }
    debug_assert!(status == napi_status_napi_ok);

    reference
}

pub fn get_reference_value(env: napi_env, reference: napi_ref) -> napi_value {
    let status: napi_status;
    let mut value: napi_value = ptr::null_mut();

    unsafe {
        status = napi_get_reference_value(env, reference, &mut value);
    }
    debug_assert!(status == napi_status_napi_ok);

    value
}

pub fn delete_reference(env: napi_env, reference: napi_ref) {
    let status: napi_status;

    unsafe {
        status = napi_delete_reference(env, reference);
    }
    debug_assert!(status == napi_status_napi_ok)
}

pub fn create_int32(env: napi_env, num: i32) -> napi_value {
    let status: napi_status;
    let mut result: napi_value = ptr::null_mut();
    unsafe {
        status = napi_create_int32(env, num, &mut result);
    }
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub struct NapiEnv {
    pub env: napi_env
}

pub fn get_typeof(env: napi_env, value: napi_value) -> napi_valuetype {
    let mut result = 0;
    let status = unsafe {
        napi_typeof(env, value, &mut result)
    };
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn get_object_map(env: napi_env, object: napi_value) -> BTreeMap<String, napi_value> {
    //get keys of object. 
    let mut map = BTreeMap::<String, napi_value>::new();
    let mut keys_value = ptr::null_mut();
    let status = unsafe {napi_get_property_names(env, object, &mut keys_value)};
    debug_assert!(status == napi_status_napi_ok);

    let mut keys_length = 0;
    let status = unsafe {napi_get_array_length(env, keys_value, &mut keys_length)};
    debug_assert!(status == napi_status_napi_ok);

    let mut key: napi_value = ptr::null_mut();

    for i in 0..keys_length {
        let mut value: napi_value = ptr::null_mut();

        let status = unsafe {napi_get_element(env, keys_value, i, &mut key)};
        debug_assert!(status == napi_status_napi_ok);
    
        let status = unsafe {napi_get_property(env, object, key, &mut value)};
        debug_assert!(status == napi_status_napi_ok);

        //TODO: what if the string length is not set right?
        if let Ok(key_string) = get_string(env, key){
            map.insert(key_string, value);
        }
    }

    map
}

pub fn slice_buffer(env: napi_env, buff: napi_value, beginning: usize, end: usize) -> napi_value {
    let mut status: napi_status;
    let mut slice_fn: napi_value = ptr::null_mut();
    let mut args: [napi_value; 2] = [ptr::null_mut(), ptr::null_mut()];
    let mut return_value: napi_value = ptr::null_mut();

    args[0] = create_int32(env, beginning as i32);
    args[1] = create_int32(env, end as i32);

    unsafe {
        status =
            napi_get_named_property(env, buff, "slice".as_ptr() as *const c_char, &mut slice_fn);
    }

    debug_assert!(status == napi_status_napi_ok);

    unsafe {
        status = napi_call_function(
            env,
            buff,
            slice_fn,
            2,
            &args[0] as *const napi_value,
            &mut return_value,
        );
    }

    debug_assert!(status == napi_status_napi_ok);

    return_value
}
