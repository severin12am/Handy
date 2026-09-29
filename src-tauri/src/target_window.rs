//! Paste into a chosen window instead of whichever app happens to be focused.
//!
//! Windows can focus that window, paste, and restore the previous focus.
//! macOS and Linux keep the matching helpers so tests run everywhere, and the
//! settings UI hides the control.

use crate::settings::DictationTarget;
use serde::Serialize;
use specta::Type;

/// A window Handy can bind to. `hwnd` is process-local and not persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub process_name: String,
    pub title: String,
    #[cfg(target_os = "windows")]
    pub hwnd: isize,
}

/// Prefer an exact process name (Telegram.exe). Among those, prefer a title
/// substring. If the process is gone, fall back to the title so a renamed
/// executable can still match. Empty fields never match the world.
pub fn select_bound_window<'a>(
    windows: &'a [WindowSnapshot],
    process_name: &str,
    title_substring: &str,
) -> Option<&'a WindowSnapshot> {
    let process = process_name.trim().to_lowercase();
    let title = title_substring.trim().to_lowercase();
    if process.is_empty() && title.is_empty() {
        return None;
    }

    let mut process_hits: Vec<&WindowSnapshot> = windows
        .iter()
        .filter(|window| !process.is_empty() && window.process_name.to_lowercase() == process)
        .collect();
    if !process_hits.is_empty() {
        if !title.is_empty() {
            if let Some(found) = process_hits
                .iter()
                .find(|window| window.title.to_lowercase().contains(&title))
            {
                return Some(*found);
            }
        }
        process_hits.sort_by(|a, b| b.title.len().cmp(&a.title.len()));
        return process_hits.first().copied();
    }

    if title.is_empty() {
        return None;
    }
    windows
        .iter()
        .find(|window| window.title.to_lowercase().contains(&title))
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct ListedWindow {
    pub process_name: String,
    pub title: String,
}

/// Result of trying to focus the bound window before a paste.
#[derive(Debug)]
pub enum FocusOutcome {
    /// No binding is active. Paste into the current focus.
    Inactive,
    /// The bound window was focused. `previous` is the HWND to restore, or 0.
    Focused { previous: isize },
    /// The window is not open. The caller must keep the text (clipboard).
    Missing,
}

pub fn focus_for_paste(target: Option<&DictationTarget>) -> FocusOutcome {
    let Some(target) = target.filter(|target| target.enabled) else {
        return FocusOutcome::Inactive;
    };
    #[cfg(target_os = "windows")]
    {
        return focus_windows_target(target);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = target;
        FocusOutcome::Inactive
    }
}

pub fn restore_focus(previous: isize) {
    #[cfg(target_os = "windows")]
    {
        if previous != 0 {
            let hwnd = windows::Win32::Foundation::HWND(previous as *mut core::ffi::c_void);
            let _ = activate_hwnd(hwnd);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = previous;
    }
}

#[cfg(target_os = "windows")]
fn focus_windows_target(target: &DictationTarget) -> FocusOutcome {
    let windows_list = list_snapshots();
    let Some(found) = select_bound_window(
        &windows_list,
        &target.process_name,
        &target.title_substring,
    ) else {
        log::warn!(
            "Bound window not found (process='{}', title='{}')",
            target.process_name,
            target.title_substring
        );
        return FocusOutcome::Missing;
    };
    let hwnd = windows::Win32::Foundation::HWND(found.hwnd as *mut core::ffi::c_void);
    let previous = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
    let previous_bits = previous.0 as isize;
    if activate_hwnd(hwnd) {
        FocusOutcome::Focused {
            previous: previous_bits,
        }
    } else {
        log::warn!("Found bound window but could not focus it");
        FocusOutcome::Missing
    }
}

#[cfg(target_os = "windows")]
pub fn list_open_windows() -> Vec<ListedWindow> {
    list_snapshots()
        .into_iter()
        .map(|window| ListedWindow {
            process_name: window.process_name,
            title: window.title,
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn list_open_windows() -> Vec<ListedWindow> {
    Vec::new()
}

#[cfg(target_os = "windows")]
pub fn capture_foreground_window() -> Option<DictationTarget> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    let title = window_title(hwnd);
    let process_name = process_name_for(hwnd);
    if title.is_empty() && process_name.is_empty() {
        return None;
    }
    Some(DictationTarget {
        process_name,
        title_substring: title,
        auto_enter: false,
        enabled: true,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn capture_foreground_window() -> Option<DictationTarget> {
    None
}

#[cfg(target_os = "windows")]
fn list_snapshots() -> Vec<WindowSnapshot> {
    use std::sync::Mutex;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::core::BOOL;
    use windows::Win32::UI::WindowsAndMessaging::EnumWindows;

    let found = Mutex::new(Vec::new());
    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let found = &*(lparam.0 as *const Mutex<Vec<WindowSnapshot>>);
        // Minimized windows are not always "visible", but they are still
        // valid bind targets (we restore them before pasting).
        let titled = window_title(hwnd);
        if !titled.is_empty() {
            let process_name = process_name_for(hwnd);
            if let Ok(mut guard) = found.lock() {
                guard.push(WindowSnapshot {
                    process_name,
                    title: titled,
                    hwnd: hwnd.0 as isize,
                });
            }
        }
        BOOL(1)
    }
    let ptr = &found as *const Mutex<Vec<WindowSnapshot>>;
    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(ptr as isize));
    }
    found.into_inner().unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn window_title(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len as usize) + 1];
        let copied = GetWindowTextW(hwnd, &mut buf);
        if copied <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..copied as usize])
    }
}

#[cfg(target_os = "windows")]
fn process_name_for(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return String::new();
        }
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        if QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_err()
        {
            return String::new();
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(target_os = "windows")]
fn activate_hwnd(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, GetForegroundWindow, GetWindowThreadProcessId, IsIconic,
        SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let foreground = GetForegroundWindow();
        let current = GetCurrentThreadId();
        let fg_thread = GetWindowThreadProcessId(foreground, None);
        let target_thread = GetWindowThreadProcessId(hwnd, None);
        // ASFW_ANY: this process may take the foreground for the paste.
        let _ = AllowSetForegroundWindow(0xFFFF_FFFF);
        if fg_thread != 0 && fg_thread != current {
            let _ = AttachThreadInput(current, fg_thread, true);
        }
        if target_thread != 0 && target_thread != current && target_thread != fg_thread {
            let _ = AttachThreadInput(current, target_thread, true);
        }
        let ok = SetForegroundWindow(hwnd).as_bool();
        if fg_thread != 0 && fg_thread != current {
            let _ = AttachThreadInput(current, fg_thread, false);
        }
        if target_thread != 0 && target_thread != current && target_thread != fg_thread {
            let _ = AttachThreadInput(current, target_thread, false);
        }
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(process: &str, title: &str) -> WindowSnapshot {
        WindowSnapshot {
            process_name: process.to_string(),
            title: title.to_string(),
            #[cfg(target_os = "windows")]
            hwnd: 0,
        }
    }

    #[test]
    fn matches_process_name_before_title() {
        let windows = vec![
            window("Cursor.exe", "Handy — community"),
            window("Telegram.exe", "Saved Messages"),
        ];
        let found = select_bound_window(&windows, "telegram.exe", "discord").unwrap();
        assert_eq!(found.process_name, "Telegram.exe");
    }

    #[test]
    fn title_breaks_ties_for_the_same_process() {
        let windows = vec![
            window("Telegram.exe", "Family"),
            window("Telegram.exe", "Work chat"),
        ];
        let found = select_bound_window(&windows, "Telegram.exe", "work").unwrap();
        assert_eq!(found.title, "Work chat");
    }

    #[test]
    fn falls_back_to_title_when_the_process_is_gone() {
        let windows = vec![window("Discord.exe", "general")];
        let found = select_bound_window(&windows, "Telegram.exe", "general").unwrap();
        assert_eq!(found.process_name, "Discord.exe");
    }

    #[test]
    fn empty_binding_matches_nothing() {
        let windows = vec![window("Telegram.exe", "Saved")];
        assert!(select_bound_window(&windows, "", "").is_none());
        assert!(select_bound_window(&windows, "Nope.exe", "missing").is_none());
    }
}
