use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::display;

pub fn execute_command(command: &str) -> io::Result<()> {
    display::display_execution_banner(command);

    // Detect shell on Windows or Unix-like systems
    let (shell_path, shell_type) = detect_shell();
    let mut cmd = Command::new(&shell_path);
    match shell_type {
        ShellType::Cmd => cmd.arg("/C").arg(command),
        ShellType::PowerShell => cmd.arg("-Command").arg(command),
        ShellType::UnixLike => cmd.arg("-c").arg(command),
        ShellType::Fish => cmd.arg("-c").arg(command),
    };

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output()?;

    display::display_stdout(&output.stdout);
    display::display_stderr(&output.stderr);
    display::display_execution_status(output.status.success());

    Ok(())
}

/// Represents different shell types with their specific command arguments
enum ShellType {
    Cmd,        // Windows Command Prompt
    PowerShell, // Windows PowerShell or PowerShell Core
    UnixLike,   // Bash, Zsh, Sh, etc.
    Fish,       // Fish shell
}

fn detect_shell() -> (String, ShellType) {
    if cfg!(target_os = "windows") {
        // On Windows, check for PowerShell first, then cmd
        let is_powershell = env::var("PSModulePath").is_ok();

        // Check if we're in PowerShell
        if is_powershell {
            // Check for PowerShell Core (pwsh.exe) first
            if let Ok(posh_path) = env::var("POSH_EXECUTABLE") {
                return (posh_path, ShellType::PowerShell);
            }

            // Otherwise use standard PowerShell
            let ps_path = env::var("COMSPEC")
                .ok()
                .and_then(|comspec| {
                    // Try to find PowerShell in the same directory as cmd.exe
                    let comspec_dir = Path::new(&comspec).parent()?;
                    let ps_path = comspec_dir.join("powershell.exe");
                    if ps_path.exists() {
                        ps_path.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "powershell.exe".to_string());

            return (ps_path, ShellType::PowerShell);
        }

        // Default to cmd.exe
        let cmd_path = env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
        (cmd_path, ShellType::Cmd)
    } else {
        // Unix-like systems
        let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // Extract shell name from path
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
