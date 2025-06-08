use os_info;
use std::env;
use std::path::Path;

/// Represents different shell types with their specific command arguments
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellType {
    Cmd,        // Windows Command Prompt
    PowerShell, // Windows PowerShell or PowerShell Core
    UnixLike,   // Bash, Zsh, Sh, etc.
    Fish,       // Fish shell
}

/// Holds information about the current system environment
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_info: String,
    pub shell_path: String,
    pub shell_type: ShellType,
}

impl SystemInfo {
    /// Detects the current system environment and returns a `SystemInfo` struct.
    pub fn new() -> Self {
        let os_info_val = os_info::get();
        let os_info = format!(
            "{} {} {}",
            os_info_val.os_type(),
            os_info_val.version(),
            os_info_val.bitness()
        );

        let (shell_path, shell_type) = detect_shell();

        SystemInfo {
            os_info,
            shell_path,
            shell_type,
        }
    }
}

/// Detects the current shell environment.
fn detect_shell() -> (String, ShellType) {
    if cfg!(target_os = "windows") {
        // On Windows, check for PowerShell first, then cmd
        if env::var("PSModulePath").is_ok() {
            // Check for PowerShell Core (pwsh.exe) first
            if let Ok(posh_path) = env::var("POSH_EXECUTABLE") {
                if Path::new(&posh_path).exists() {
                    return (posh_path, ShellType::PowerShell);
                }
            }
            // Otherwise use standard PowerShell
            return ("powershell.exe".to_string(), ShellType::PowerShell);
        }
        // Default to cmd.exe
        (
            env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string()),
            ShellType::Cmd,
        )
    } else {
        // Unix-like systems
        let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let shell_name = Path::new(&shell_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sh")
            .to_lowercase();

        if shell_name == "fish" {
            (shell_path, ShellType::Fish)
        } else {
            (shell_path, ShellType::UnixLike)
        }
    }
}
