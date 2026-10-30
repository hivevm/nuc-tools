//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ADR-draft-functional-core).

use std::process::ExitCode;

/// Exit code of a run that could not decide: nothing is implemented to decide with yet, so every
/// command fails closed (Q-1).
const UNDECIDED: u8 = 2;

/// The entry point of the binary: runs the command named on the command line and returns the
/// exit code the process ends with.
pub fn main() -> ExitCode {
    eprintln!("nuc: no command is implemented yet");
    ExitCode::from(UNDECIDED)
}
