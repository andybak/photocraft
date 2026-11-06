//! SteamVR's Xwayland laser mouse stops forwarding clicks after a text field takes focus.

use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt as _;

/// Re-execute through gamescope's native Wayland socket before threads or application state exist.
/// `exec` retains the Steam-launched process instead of leaving a detached child for Steam to lose.
pub fn prefer_native_wayland() {
    let Some(display) = native_wayland_display(|name| std::env::var_os(name)) else {
        return;
    };
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("PhotoCraft could not select native Wayland for SteamVR: {error}");
            return;
        }
    };
    let error = std::process::Command::new(executable)
        .args(std::env::args_os().skip(1))
        .env("WAYLAND_DISPLAY", display)
        .env("PHOTOCRAFT_NATIVE_WAYLAND", "1")
        .env_remove("DISPLAY")
        .exec();
    eprintln!("PhotoCraft could not restart on native Wayland for SteamVR: {error}");
}

fn native_wayland_display(mut var: impl FnMut(&str) -> Option<OsString>) -> Option<OsString> {
    let set = |value: Option<&OsStr>| value.is_some_and(|value| !value.is_empty());
    if var("StreamForOpenVR").as_deref() != Some(OsStr::new("1")) || set(var("WAYLAND_DISPLAY").as_deref()) || set(var("WAYLAND_SOCKET").as_deref()) {
        return None;
    }
    var("GAMESCOPE_WAYLAND_DISPLAY").filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;

    use super::native_wayland_display;

    fn display(values: &[(&str, &str)]) -> Option<OsString> {
        let values: HashMap<_, _> = values.iter().copied().collect();
        native_wayland_display(|name| values.get(name).map(OsString::from))
    }

    #[test]
    fn steamvr_uses_gamescope_native_wayland() {
        assert_eq!(display(&[("StreamForOpenVR", "1"), ("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0"), ("DISPLAY", ":1")]), Some(OsString::from("gamescope-0")));
    }

    #[test]
    fn ordinary_and_already_native_sessions_do_not_reexec() {
        assert_eq!(display(&[("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0")]), None);
        assert_eq!(display(&[("StreamForOpenVR", "1"), ("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0"), ("WAYLAND_DISPLAY", "wayland-0")]), None);
        assert_eq!(display(&[("StreamForOpenVR", "1"), ("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0"), ("WAYLAND_SOCKET", "3")]), None);
        assert_eq!(display(&[("StreamForOpenVR", "1"), ("GAMESCOPE_WAYLAND_DISPLAY", "")]), None);
        assert_eq!(display(&[("StreamForOpenVR", "0"), ("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0")]), None);
        assert_eq!(display(&[("StreamForOpenVR", "true"), ("GAMESCOPE_WAYLAND_DISPLAY", "gamescope-0")]), None);
    }
}
