use crate::util::fetch_text;
use color_eyre::eyre::ContextCompat;
use std::sync::Mutex;

static BETTER_FOX_USER_JS: Mutex<&'static str> = Mutex::new("");

pub fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    // We are holding this lock across this request because we don't want
    // another thread to try to simultaneously fetch the resource
    let mut lock = BETTER_FOX_USER_JS.lock().ok().context("Lock was poisoned")?;
    if lock.is_empty() {
        let s = fetch_text(
            "Betterfox User.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js"
        )?;
        // SAFETY: This will only happen once during a program execution, and we really don't want to clone this string.
        *lock = String::leak(s);
    }

    Ok(*lock)
}
