/// Check if the current process has elevated/admin privileges.
#[allow(dead_code)]
pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        // On Windows, attempt a privileged operation to check
        use std::process::Command;
        Command::new("net")
            .args(["session"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        false
    }
}
