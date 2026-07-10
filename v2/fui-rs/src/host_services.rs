use crate::event;

pub fn host_service_result_buffer_ptr() -> *mut u8 {
    event::__fui_text_buffer() as *mut u8
}

pub fn host_service_result_buffer_size() -> u32 {
    event::__fui_text_buffer_size()
}

fn assert_result_byte_length(result_len: u32, import_name: &str) {
    let capacity = host_service_result_buffer_size();
    if result_len > capacity {
        panic!(
            "Host service {} returned {} bytes but the shared result buffer only holds {}.",
            import_name, result_len, capacity
        );
    }
}

pub fn decode_host_service_string_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> String {
    assert_result_byte_length(result_len, import_name);
    if result_len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(result_ptr as *const u8, result_len as usize) };
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn decode_host_service_bytes_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<u8> {
    assert_result_byte_length(result_len, import_name);
    if result_len == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(result_ptr as *const u8, result_len as usize) }.to_vec()
}

pub fn decode_host_service_i32_array_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<i32> {
    assert_result_byte_length(result_len, import_name);
    assert_eq!(
        result_len & 3,
        0,
        "Host service {} returned misaligned i32 array byte length.",
        import_name
    );
    if result_len == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(result_ptr as *const i32, (result_len >> 2) as usize).to_vec()
    }
}

pub fn decode_host_service_u32_array_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<u32> {
    assert_result_byte_length(result_len, import_name);
    assert_eq!(
        result_len & 3,
        0,
        "Host service {} returned misaligned u32 array byte length.",
        import_name
    );
    if result_len == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(result_ptr as *const u32, (result_len >> 2) as usize).to_vec()
    }
}

pub fn decode_host_service_i64_array_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<i64> {
    assert_result_byte_length(result_len, import_name);
    assert_eq!(
        result_len & 7,
        0,
        "Host service {} returned misaligned i64 array byte length.",
        import_name
    );
    if result_len == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(result_ptr as *const i64, (result_len >> 3) as usize).to_vec()
    }
}

pub fn decode_host_service_u64_array_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<u64> {
    assert_result_byte_length(result_len, import_name);
    assert_eq!(
        result_len & 7,
        0,
        "Host service {} returned misaligned u64 array byte length.",
        import_name
    );
    if result_len == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(result_ptr as *const u64, (result_len >> 3) as usize).to_vec()
    }
}

pub fn decode_host_service_f64_array_result(
    result_ptr: *mut u8,
    result_len: u32,
    import_name: &str,
) -> Vec<f64> {
    assert_result_byte_length(result_len, import_name);
    assert_eq!(
        result_len & 7,
        0,
        "Host service {} returned misaligned f64 array byte length.",
        import_name
    );
    if result_len == 0 {
        return Vec::new();
    }
    unsafe {
        std::slice::from_raw_parts(result_ptr as *const f64, (result_len >> 3) as usize).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_string_result_from_shared_text_buffer() {
        let ptr = host_service_result_buffer_ptr();
        let bytes = b"demo-host";
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        }
        assert_eq!(
            decode_host_service_string_result(ptr, bytes.len() as u32, "demo_service"),
            "demo-host"
        );
    }
}
