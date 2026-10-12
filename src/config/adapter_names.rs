#[cfg(windows)]
pub fn get_names() -> std::collections::HashMap<String, String> {
    use std::{collections::HashMap, ffi::CStr, ptr, slice};
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GAA_FLAG_INCLUDE_ALL_INTERFACES, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
    };

    let mut size = 0u32;

    let status = unsafe {
        GetAdaptersAddresses(
            0,
            GAA_FLAG_INCLUDE_ALL_INTERFACES,
            ptr::null(),
            ptr::null_mut(),
            &mut size,
        )
    };

    if status != 111 || size == 0 {
        return HashMap::new();
    }

    for attempt in 0..3 {
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];

        let status = unsafe {
            GetAdaptersAddresses(
                0,
                GAA_FLAG_INCLUDE_ALL_INTERFACES,
                ptr::null(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };

        if status == 111 {
            if attempt == 2 {
                eprintln!("GetAdaptersAddresses: буфер не удалось выделить за 3 попытки");
                return HashMap::new();
            }

            continue;
        }

        if status != 0 {
            eprintln!("GetAdaptersAddresses failed: {status}");
            return HashMap::new();
        }

        let mut names = HashMap::new();
        let mut adapter = buffer.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();

        while !adapter.is_null() {
            let item = unsafe { &*adapter };

            if !item.AdapterName.is_null() && !item.FriendlyName.is_null() {
                let guid = unsafe {
                    CStr::from_ptr(item.AdapterName.cast())
                        .to_string_lossy()
                        .trim_matches(['{', '}'])
                        .to_ascii_uppercase()
                };

                let name = unsafe {
                    let mut len = 0;

                    while *item.FriendlyName.add(len) != 0 {
                        len += 1;
                    }

                    String::from_utf16_lossy(slice::from_raw_parts(item.FriendlyName, len))
                };

                names.insert(guid, name);
            }

            adapter = item.Next;
        }

        return names;
    }
    HashMap::new()
}

#[cfg(windows)]
pub fn get_name(
    pcap_name: &str,
    names: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let guid = pcap_name
        .rsplit_once('{')?
        .1
        .trim_end_matches('}')
        .to_ascii_uppercase();

    names.get(&guid).cloned()
}

#[cfg(not(windows))]
pub fn get_names() -> () {}

#[cfg(not(windows))]
pub fn get_name(pcap_name: &str, _: &()) -> Option<String> {
    Some(pcap_name.to_string())
}
