//! Memory footprint auditing utilities.
//!
//! Measures resident set size (RSS) to verify that the daemon operates
//! within the ~3-5 MB memory footprint target.

/// Returns the current process resident set size (RSS) in kilobytes, if available.
pub fn get_resident_memory_kb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            let mut parts = statm.split_whitespace();
            let _size = parts.next();
            if let Some(rss_pages_str) = parts.next() {
                if let Ok(rss_pages) = rss_pages_str.parse::<usize>() {
                    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
                    let page_size_kb = if page_size > 0 {
                        (page_size as usize) / 1024
                    } else {
                        4
                    };
                    return Some(rss_pages * page_size_kb);
                }
            }
        }
    }

    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } == 0 {
            let usage = unsafe { usage.assume_init() };
            #[cfg(target_os = "macos")]
            {
                // On macOS, ru_maxrss is in bytes
                return Some((usage.ru_maxrss as usize) / 1024);
            }
            #[cfg(not(target_os = "macos"))]
            {
                // On Linux/BSD, ru_maxrss is in kilobytes
                return Some(usage.ru_maxrss as usize);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resident_memory() {
        let mem = get_resident_memory_kb();
        assert!(
            mem.is_some(),
            "Memory measurement should be available on Unix"
        );
        let kb = mem.unwrap();
        // A minimal Rust test process should be well under 100 MB
        assert!(kb > 0 && kb < 100_000, "RSS memory was {} KB", kb);
    }
}
