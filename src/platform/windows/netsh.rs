use std::io;
use std::os::windows::process::CommandExt;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub fn set_interface_metric(index: u32, metric: u16) -> io::Result<()> {
    let cmd = format!(
        "netsh interface ip set interface {} metric={}",
        index, metric
    );
    exe_cmd(&cmd)
}
pub fn exe_cmd(cmd: &str) -> io::Result<()> {
    let out = std::process::Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW)
        .arg("/C")
        .arg(cmd)
        .output()?;
    if !out.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("cmd={:?},out={:?}", cmd, String::from_utf8(out.stderr)),
        ));
    }
    Ok(())
}
