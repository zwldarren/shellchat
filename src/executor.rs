use crate::display;
use crate::system::{ShellType, SystemInfo};
use std::io;
use std::process::{Command, Stdio};

pub fn execute_command(command: &str, system_info: &SystemInfo) -> io::Result<()> {
    let mut cmd = Command::new(&system_info.shell_path);
    match system_info.shell_type {
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
