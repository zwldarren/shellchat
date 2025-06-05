use std::io;
use std::process::{Command, Stdio};

use crate::display;

pub fn execute_command(command: &str) -> io::Result<()> {
    display::display_execution_banner(command);

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output()?;

    display::display_stdout(&output.stdout);
    display::display_stderr(&output.stderr);
    display::display_execution_status(output.status.success());

    Ok(())
}
