use crate::{
    ARGS, Args, brave, brave::Brave, browser::{Browser, installation::Installation}
};
use color_eyre::eyre::Context;
use inquire::error::InquireResult;
use serde_json::{Map, Value};
use std::{
    collections::HashSet, fmt::Display, fs, io::{Read, stdin}, path::{Path, PathBuf}, process, process::Stdio
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tracing::{debug, debug_span, info, warn};

pub mod archive;
pub mod logging;

pub fn get_or_insert_obj<'a>(
    map: &'a mut Map<String, Value>,
    key: &str
) -> Option<&'a mut Map<String, Value>> {
    let ret = map
        .entry(key.to_owned())
        .or_insert_with(|| {
            debug!("Inserting {key}");
            Value::Object(serde_json::Map::new())
        })
        .as_object_mut();

    if ret.is_none() {
        debug!("Failed to cast to object");
    }

    ret
}

pub fn roaming_data_base() -> Option<PathBuf> {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_dir()
    } else {
        dirs::home_dir()
    }
}

pub fn local_data_base() -> Option<PathBuf> {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_local_dir()
    } else {
        dirs::config_local_dir()
    }
}

pub fn local_app_bases() -> impl Iterator<Item = PathBuf> {
    if cfg!(target_os = "windows") {
        vec![
            Some(PathBuf::from("C:\\Program Files")),
            Some(PathBuf::from("C:\\Program Files (x86)")),
        ]
    } else if cfg!(target_os = "macos") {
        vec![Some(PathBuf::from("/Applications")), dirs::home_dir().map(|p| p.join("Applications"))]
    } else {
        vec![Some(PathBuf::from("/opt"))]
    }
    .into_iter()
    .flatten()
}

#[rustfmt::skip]
pub fn local_snap_base() -> Option<PathBuf> {
    if cfg!(target_os = "linux") {
        dirs::home_dir().map(|p| p.join("snap"))
    } else {
        None
    }
}

#[rustfmt::skip]
pub fn flatpak_base() -> Option<PathBuf> {
    if cfg!(target_os = "linux") {
        dirs::home_dir().map(|p| p.join(".var/app"))
    } else {
        None
    }
}

// 202501192003
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d%H%M").to_string()
}

pub fn fetch_text(name: &str, url: &str) -> color_eyre::Result<String> {
    ureq::get(url)
        .call()
        .wrap_err_with(|| format!("Failed to request {name}"))?
        .into_body()
        .read_to_string()
        .wrap_err_with(|| format!("Failed to read {name} to string"))
}

pub fn validate_profile_dir(profile: &Path) -> bool {
    if !profile.exists() {
        debug!(path = %profile.display(), "Profile does not exist");
        return false;
    }

    let children = match fs::read_dir(profile) {
        Ok(c) => c,
        Err(why) => {
            debug!(path = %profile.display(), err = ?why, "Failed to read profile directory");
            return false;
        }
    };

    let children = children
        .into_iter()
        .filter_map(Result::ok)
        .filter(|c| c.file_type().map(|f| f.is_file() || f.is_dir()).unwrap_or(false))
        .count();

    // If no files or only times.json (on Firefox)
    if children <= 3 {
        debug!(path = %profile.display(), "Profile directory is empty or only contains times.json");
        return false;
    }

    true
}

pub fn select_profiles<P: Display, B: Browser>(mut profiles: Vec<P>, selected: &[usize]) -> Vec<P> {
    if args().auto_confirm {
        profiles
    } else if profiles.len() == 1 {
        vec![profiles.remove(0)]
    } else {
        inquire::MultiSelect::new(
            &format!("Which profiles to debloat for {}?", B::name()),
            profiles
        )
        .with_default(selected)
        .prompt()
        .unwrap_or_exit()
        .into_iter()
        .collect::<Vec<_>>()
    }
}

fn get_matching_running_processes(system: &mut System, name: &str) -> String {
    let lower_name = name.to_lowercase();
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::default()));
    let processes = system.processes();

    debug!("Found {} processes total", processes.len());

    let running_instances = processes
        .values()
        .filter_map(|p| {
            let name = p.name().to_str()?;
            name.to_lowercase().contains(&lower_name).then_some(name)
        })
        .collect::<HashSet<_>>();

    running_instances.into_iter().collect::<Vec<_>>().join(", ")
}

pub fn check_if_running(system: &mut System, browser_name: &str) {
    if args().auto_confirm {
        return;
    }

    let processes = get_matching_running_processes(system, browser_name);
    if processes.is_empty() {
        return;
    }

    warn!("Please close all instances before debloating ({processes})");
    info!("Press enter to continue");
    if let Err(why) = stdin().read_exact(&mut [0_u8]) {
        warn!(err = ?why, "Error reading stdin, exiting");
        process::exit(1);
    }

    let processes = get_matching_running_processes(system, browser_name);
    if processes.is_empty() {
        return;
    }

    warn!("Some processes are still running ({processes})");

    let should_continue = inquire::prompt_confirmation("Continue anyway? (y/n)").unwrap_or_exit();
    if !should_continue {
        process::exit(1);
    }
}

pub fn start_fetch_resource<F, O>(f: F, browser_name: &'static str)
where
    F: Fn() -> color_eyre::Result<O> + Send + 'static
{
    std::thread::spawn(move || {
        let span = debug_span!("fetching resources for", name = %browser_name);
        let _enter = span.enter();

        match f() {
            Ok(_) => debug!("Fetched resources"),
            Err(why) => warn!(err = ?why, "Failed to fetch resources")
        }
    });
}

pub trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T> UnwrapOrExit<T> for InquireResult<T> {
    fn unwrap_or_exit(self) -> T {
        self.unwrap_or_else(|_| {
            warn!("User killed program");
            process::exit(1);
        })
    }
}

// Just for usage when doing a mass JSON insertion like `brave::preferences`
#[macro_export]
macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}

pub struct RenderedBrowser {
    pub installations: Vec<Installation>,
    pub fetch_resources: Option<fn() -> color_eyre::Result<&'static str>>,
    pub name: &'static str
}

#[macro_export]
macro_rules! render_browsers {
    ($($browser:ty),+) => {{
        vec![$(
            $crate::RenderedBrowser {
                installations: <$browser>::installations(),
                fetch_resources: <$browser>::fetch_resources(),
                name: <$browser>::name()
            },
        )+]
    }};
}

pub fn args() -> &'static crate::Args {
    ARGS.get().expect("to be initialized")
}

#[cfg(target_os = "windows")]
pub fn elevate_and_run(flag: &str) -> color_eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let exe_path = std::env::current_exe().wrap_err("failed to resolve current exe")?;

    let mut elevated_args = vec![flag.to_string()];
    if args.len() > 1 {
        elevated_args.extend_from_slice(&args[1..]);
    }

    let ps_script = format!(
        "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait",
        exe_path.display(),
        elevated_args.join("','")
    );

    let status = std::process::Command::new("powershell")
        .args(&["-Command", &ps_script])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .status()
        .wrap_err("Failed to run powershell")?;

    if !status.success() {
        color_eyre::eyre::bail!("got non-zero exit code: {status}");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn elevate_and_run(flag: &str) -> color_eyre::Result<()> {
    let mut args: Vec<_> = std::env::args().collect();
    if let Some(absolute_path) = std::env::current_exe().ok().as_deref().and_then(Path::to_str) {
        args[0] = absolute_path.to_string();
    }

    args.push(flag.to_string());

    let status = process::Command::new("sudo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .status()
        .wrap_err("Failed to run powershell")?;

    if !status.success() {
        color_eyre::eyre::bail!("got non-zero exit code: {status}");
    }

    Ok(())
}

// TODO this is a mess, clean up
pub fn process_single_policies(args: &Args, installations: &[&Installation], post: bool) {
    if !args.policies && !args.windows_brave_policies && !args.linux_brave_policies && !args.linux_firefox_policies {
        return;
    }

    // Short circuit, this happens after we run this as a child with elevated permissions
    #[cfg(target_os = "windows")]
    if post || args.windows_brave_policies {
        let explicit = args.windows_brave_policies;
        let install = installations.iter().find(|i| i.browser_name == Brave::name());

        if let Some(install) = install {
            try_policies_or_fail(
                brave::create_policies_windows(install, args.backup, explicit),
                explicit
            );
        } else if explicit {
            warn!("no brave installation found");
        }
    }

    #[cfg(target_os = "linux")]
    if post || args.linux_brave_policies {
        let explicit = args.linux_brave_policies;
        if installations.iter().any(|i| i.browser_name == Brave::name()) {
            try_policies_or_fail(
                brave::create_policies_linux(args.backup, explicit),
                explicit
            );
        } else if explicit {
            warn!("no brave installation found");
        }
    }

    #[cfg(target_os = "linux")]
    if post || args.linux_firefox_policies {
        let explicit = args.linux_firefox_policies;
        if installations.iter().any(|i| i.browser_name == "Firefox") {
            try_policies_or_fail(
                crate::firefox::create_linux_policies_file(args.backup, explicit),
                explicit
            );
        } else if explicit {
            warn!("no firefox installation found");
        }
    }
}

fn try_policies_or_fail(r: color_eyre::Result<()>, explicit: bool) {
    if let Err(why) = r {
        warn!(err = ?why, "Failed to create policies");
        if explicit {
            process::exit(1);
        }
    } else {
        info!("Successfully created policies");
    }
}

pub fn should_elevate() -> bool {
    if args().auto_confirm {
        return true;
    };

    info!("Policy creation requires elevated permissions.");
    inquire::prompt_confirmation("Request elevated permissions? (y/n)").unwrap_or_exit()
}
