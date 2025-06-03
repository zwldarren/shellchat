use clap::Parser;
use std::process::{Command, Stdio};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The command to execute
    #[arg(short, long)]
    command: String,

    /// Arguments for the command
    #[arg(last = true)]
    args: Vec<String>,
}

fn execute_command(command: &str, args: &[String]) -> io::Result<()> {
    let mut cmd = Command::new(command);
    cmd.args(args);

    // Capture stdout and stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output()?;

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    execute_command(&args.command, &args.args)?;

    Ok(())
}
