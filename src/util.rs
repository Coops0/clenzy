// pub static CONFIG_DIRS: LazyLock<HashSet<PathBuf>> = LazyLock::new(|| {
//     [dirs::config_dir(), dirs::config_local_dir(), dirs::data_dir(), dirs::data_local_dir()]
//         .into_iter()
//         .flatten()
//         .collect::<HashSet<PathBuf>>()
// });

use std::path::PathBuf;
use serde_json::{Map, Value};
use tracing::debug;

pub static mut BROWSER_FOUND: bool = false;

#[macro_export]
macro_rules! try_browser {
    ($browser:expr, $path:path, $debloat:path) => {
        if let Some(p) = $path() {
            if inquire::prompt_confirmation(format!(
                "found {} at {}, continue?",
                $browser,
                p.display()
            ))
            .unwrap_or_default()
            {
                unsafe {
                    util::BROWSER_FOUND = true;
                }

                $debloat(&p)?;
                tracing::info!("Debloated {}", $browser);
            }
        }
    };
}

pub fn get_or_insert_obj<'a>(
    map: &'a mut Map<String, Value>,
    key: &str,
) -> Option<&'a mut Map<String, Value>> {
    map.entry(key.to_string())
        .or_insert_with(|| {
            debug!("creating {key}");
            Value::Object(serde_json::Map::new())
        })
        .as_object_mut()
}

pub fn any_browser_found() -> bool {
    unsafe { BROWSER_FOUND }
}

pub fn config_base() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return dirs::config_dir();
    #[cfg(target_os = "windows")]
    return dirs::data_dir();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return dirs::config_local_dir();
}