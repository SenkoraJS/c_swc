use std::ffi::c_char;

use errors::Error;

mod errors;
mod transpiler;

/// # Safety
///
/// This function should be save if the `file_name` and `content` are valid strings.
#[no_mangle]
pub unsafe extern "C" fn transpile_module(file_name: *mut c_char, content: *mut c_char) -> i8 {
    let file_name = match std::ffi::CStr::from_ptr(file_name).to_str() {
        Ok(file_name) => file_name,
        Err(_) => return Error::InvalidString.into(),
    };
    let content = match std::ffi::CStr::from_ptr(content).to_str() {
        Ok(content) => content,
        Err(_) => return Error::InvalidString.into(),
    };

    let result = transpiler::transpile_module(file_name.to_string(), content.to_string());

    match result {
        Ok(_) => 0,
        Err(error) => error.into(),
    }
}
