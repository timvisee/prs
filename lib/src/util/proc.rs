#[cfg(target_os = "linux")]
pub fn pids_with_file_open(path: &std::path::Path) -> std::io::Result<Vec<nix::unistd::Pid>> {
    use nix::libc::pid_t;
    use nix::unistd::Pid;
    use procfs::process::all_processes;

    let mut pids = vec![];

    let target = path.canonicalize()?;

    for process in all_processes().unwrap() {
        let Ok(process) = process else {
            continue;
        };

        let Ok(fds) = process.fd() else {
            continue;
        };

        for fd in fds {
            let Ok(fd) = fd else {
                continue;
            };

            if matches!(fd.target, procfs::process::FDTarget::Path(path) if path == target) {
                pids.push(Pid::from_raw(process.pid as pid_t));
            }
        }
    }

    Ok(pids)
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub fn pids_with_file_open(path: &std::path::Path) -> std::io::Result<Vec<nix::unistd::Pid>> {
    use nix::libc::pid_t;
    use nix::unistd::Pid;
    use std::process::Command;

    // Use `lsof -t <file>` to get PIDs which is more portable across Unix systems
    let output = Command::new("lsof")
        .arg("-t")
        .arg(path.to_string_lossy().as_ref())
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();
    for line in stdout.lines() {
        if let Ok(pid) = line.trim().parse::<pid_t>() {
            pids.push(Pid::from_raw(pid));
        }
    }

    Ok(pids)
}

#[cfg(target_os = "linux")]
pub fn cmdline(pid: nix::unistd::Pid) -> std::io::Result<String> {
    use std::fs;

    let cmdline = fs::read_to_string(format!("/proc/{pid}/cmdline"))?;
    Ok(cmdline.replace('\0', " ").trim().to_string())
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub fn cmdline(pid: nix::unistd::Pid) -> std::io::Result<String> {
    use std::process::Command;

    let output = Command::new("ps")
        .arg("-p")
        .arg(pid.as_raw().to_string())
        .arg("-o")
        .arg("command=")
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other("Failed to get process command line"));
    }

    let cmdline = String::from_utf8_lossy(&output.stdout);
    Ok(cmdline.trim().to_string())
}
