/// Runs a command and returns an error if the command fails, just convenience for users.
#[doc(hidden)]
#[allow(dead_code)]
pub fn run_command(command: &str, args: &[&str]) -> std::io::Result<Vec<u8>> {
    let full_cmd = format!("{} {}", command, args.join(" "));
    log::debug!("Running command: \"{full_cmd}\"...");
    let out = match std::process::Command::new(command).args(args).output() {
        Ok(out) => out,
        Err(e) => {
            log::error!("Run command: \"{full_cmd}\" failed with: {e}");
            return Err(e);
        }
    };
    if !out.status.success() {
        let err = String::from_utf8_lossy(if out.stderr.is_empty() {
            &out.stdout
        } else {
            &out.stderr
        });
        let info = format!("Run command: \"{full_cmd}\" failed with {err}");
        log::error!("{}", info);
        return Err(std::io::Error::other(info));
    }
    Ok(out.stdout)
}
