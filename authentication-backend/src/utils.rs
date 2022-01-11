use cfg_if::cfg_if;
use regex::Regex;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}


pub fn validate_username(username: &str) -> bool {
    let username_re = unwrap_res_abort(Regex::new(r"[1-9a-z_\.\-]{3,15}$"));
    username_re.is_match(username)
}

#[inline]
pub fn unwrap_abort<T>(o: Option<T>) -> T {
    use std::process;
    match o {
        Some(t) => t,
        None => process::abort(),
    }
}

#[inline]
pub fn unwrap_res_abort<T, E>(o: Result<T, E>) -> T {
    use std::process;
    match o {
        Ok(t) => t,
        Err(_) => process::abort(),
    }
}