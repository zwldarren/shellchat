use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn execute_command(command: &str) -> io::Result<()> {
    println!("\nExecuting: {}", command);
    println!("{}", "-".repeat(40));

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

    if !output.stdout.is_empty() {
        io::stdout().write_all(&output.stdout)?;
    }

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr)?;
    }

    println!("{}", "-".repeat(40));
    Ok(())
}
