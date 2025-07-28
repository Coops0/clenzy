use crate::util::fetch_text;
use color_eyre::eyre::ContextCompat;
use std::sync::Mutex;

static BETTER_ZEN_USER_JS: Mutex<&'static str> = Mutex::new("");

pub fn get_better_zen_user_js() -> color_eyre::Result<&'static str> {
    let mut lock = BETTER_ZEN_USER_JS.lock().ok().wrap_err("Lock was poisoned")?;
    if lock.is_empty() {
        let s = fetch_text(
            "Better Zen user.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/refs/heads/main/zen/user.js"
        )?;
        *lock = String::leak(s);
    }

    Ok(*lock)
}
