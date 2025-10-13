pub mod prelude;
pub use redshell_macros;

use std::process::Command;
use std::os::fd::AsRawFd;
use std::os::fd::IntoRawFd;
use std::fmt;
use std::rc::{Rc, Weak};
use std::cell::Cell;
use std::io::{Read, Write, PipeReader, PipeWriter};
use std::os::fd::OwnedFd;
use std::path::Path;
use nix::sys::memfd;


/// Create a command.
/// cmd!("curl https://myhostname");

/// Spawn one or more commands.
/// spawn!("curl https://myhostname");
/// spawn!(cmd1);
/// spawn!(cmd1, cmd2, ..);

/// Wait for one or more commands to finish.
/// wait!(cmd1);
/// wait!(cmd1, cmd2, ..);

/// Exec one or more commands and wait until they all returns.
/// exec!(cmd1);
/// exec!(cmd1, cmd2, ..);

/// Execute one or more commands and wait until they all returns.
#[macro_export]
macro_rules! exec {
    ($expression:expr) => {
        let cmd = $crate::redshell_macros::cmd!($expression);
        match cmd.status() {
            Ok(status) => {
                return status;
            },
            Err(e) => {
                eprintln!("Command not found: {e}");
                return 1;
            }
        }
    };
}

pub trait CommandExt {
    fn redirect<S: AsRef<Path>>(&mut self, fd: i32, file: S);
}

pub struct CommandInfo {
    command: std::process::Command,
}

pub struct MemoryFile {
    memfd: OwnedFd,
}

pub struct RdPipe {
    rdend: OwnedFd,
}

pub struct WrPipe {
    wrend: OwnedFd,
}

impl CommandExt for Command {
    /**
     * Open file with the specified path at the specified fd.
     * FIXME:
     *   -read/write => should be configurable
     *   -truncate => should be configurable
     */
    fn redirect<S: AsRef<Path>>(&mut self, fd: i32, path: S) {
        use std::os::unix::process::CommandExt;

        // we need a deep-copy to share this variable with pre_exec()
        let path = path.as_ref().to_path_buf();
        unsafe {
            eprintln!("pre_exec(Redirect {:?} to {fd})", path);
            self.pre_exec(move || {
                eprintln!("Redirect {path:?} to {fd}");
                let Ok(file) = std::fs::OpenOptions::new()
                    //.write(false)
                    //.create_new(true)
                    //.truncate(true)
                    .open(&path)
                else {
                    eprintln!("Failed to open {:?}", path);
                    return Ok(());
                };

                if let Err(e) = nix::unistd::dup2_raw(file, fd) {
                    panic!("Failed to redirect {path:?} to {fd}: {e}");
                }

                Ok(())
            });
        }
    }
}

pub fn pipe() -> (RdPipe, WrPipe) {
    let (rdend, wrend) = nix::unistd::pipe()
        .expect("pipe() failed");

    let rdpipe = RdPipe {
        rdend,
    };
    let wrpipe = WrPipe {
        wrend,
    };
    (rdpipe, wrpipe)
}

impl Read for RdPipe {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let rt = nix::unistd::read(&self.rdend, buf)?;
        Ok(rt)
    }
}

impl fmt::Display for RdPipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fd = self.rdend.as_raw_fd();
        write!(f, "/dev/fd/{fd}")
    }
}

impl WrPipe {
    pub fn get_filename(&self) -> String {
        let fd = self.wrend.as_raw_fd();
        format!("/dev/fd/{fd}")
    }
}

impl Write for WrPipe {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let rt = nix::unistd::write(&self.wrend, buf)?;
        Ok(rt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for WrPipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fd = self.wrend.as_raw_fd();
        write!(f, "/dev/fd/{fd}")
    }
}

impl MemoryFile {
    pub fn new() -> Self {
        let memfd = memfd::memfd_create("redshell", memfd::MFdFlags::empty())
            .expect("memfd_create() failed");
        Self {
            memfd,
        }
    }

    pub fn read_string(&mut self) -> String {
        let mut s = String::new();
        self.read_to_string(&mut s);
        return s;
    }
}

impl From<&str> for MemoryFile {
    fn from(buf: &str) -> MemoryFile {
        let mut memfd = MemoryFile::new();
        write!(memfd, "{buf}");
        memfd
    }
}

impl From<String> for MemoryFile {
    fn from(buf: String) -> MemoryFile {
        let mut memfd = MemoryFile::new();
        write!(memfd, "{buf}");
        memfd
    }
}

impl Read for MemoryFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let rt = nix::unistd::read(&self.memfd, buf)?;
        Ok(rt)
    }
}

impl Write for MemoryFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let rt = nix::unistd::write(&self.memfd, buf)?;
        Ok(rt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for MemoryFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fd = self.memfd.as_raw_fd();
        write!(f, "/dev/fd/{fd}")
    }
}

#[cfg(test)]
mod tests {
    use crate as redshell;
    use redshell::prelude::*;
    use std::io::Read;

    #[test]
    fn test_expand_cmd2_basic() {
        //CommandInfo::new(
    }

    #[test]
    fn test_expand_cmd_basic() {
        let cmd = cmd!("make -C src/ all");
        let mut expected = std::process::Command::new("make");
        expected.arg("-C");
        expected.arg("src/");
        expected.arg("all");
        assert_eq!(format!("{cmd:?}"), format!("{expected:?}"));
    }

    #[test]
    fn test_expand_cmd_with_var() {
        let dir = "src/";
        let c1 = "src";
        let c2 = "/";
        let target = "all";

        let mut expected = std::process::Command::new("make");
        expected.arg("-C");
        expected.arg("src/");
        expected.arg("all");

        let cmd = cmd!("make -C {dir} all");
        assert_eq!(format!("{cmd:?}"), format!("{expected:?}"));

        let cmd = cmd!("make -C {c1}{c2} {target}");
        assert_eq!(format!("{cmd:?}"), format!("{expected:?}"));

        let cmd = cmd!(" make -C '{c1}{c2}' {target}");
        assert_eq!(format!("{cmd:?}"), format!("{expected:?}"));

        let cmd = cmd!("\t make -C '{c1}{c2}' {target}\n");
        assert_eq!(format!("{cmd:?}"), format!("{expected:?}"));
    }

    #[test]
    fn test_simple_pipe() {
        let (mut rdpipe, wrpipe) = redshell::pipe();
        let mut cmd = cmd!("dd if=/proc/cpuinfo of={wrpipe}");
        let mut child = cmd.spawn().unwrap();


        /*
        child.wait().unwrap();
        let mut cpuinfo = String::new();
        rdpipe.read_to_string(&mut cpuinfo).unwrap();
        */

        let mut cpuinfo = vec![0;128];
        let len = rdpipe.read(&mut cpuinfo).unwrap();
        assert!(len > 0);
    }

    #[test]
    fn test_memory_file() {
        let content = "Hello World !";
        let input = redshell::MemoryFile::from(content);
        let mut output = redshell::MemoryFile::new();
        let mut cmd = cmd!("dd if={input} of={output}");
        let status = cmd.status().unwrap();

        assert_eq!(status.code(), Some(0));
        assert_eq!(output.read_string(), content);
    }

    #[test]
    fn test_redirection() {
        let content = "Hello World !";
        let input = redshell::MemoryFile::from(content);
        let mut output = redshell::MemoryFile::new();
        let mut cmd = cmd!("dd if={input} >{output}");
        let status = cmd.status().unwrap();

        assert_eq!(status.code(), Some(0));
        assert_eq!(output.read_string(), content);
    }

    /*
    #[test]
    fn test_simple_pipe_poc() {
        use nix::unistd::pipe;
        use std::os::fd::AsRawFd;

        let (rdend, wrend) = pipe().unwrap();

        let rdfile = format!("/dev/fd/{}", rdend.as_raw_fd());
        let wrfile = format!("/dev/fd/{}", wrend.as_raw_fd());

        let mut cmd1 = cmd!("dd if=/proc/cpuinfo of={wrfile}");
        let mut cmd2 = cmd!("cat {rdfile}");

        let mut child1 = cmd1.spawn().unwrap();
        let mut child2 = cmd2.spawn().unwrap();


        child1.wait().unwrap();
        child2.wait().unwrap();
    }
    */

    #[test]
    fn test_redirect_stdout() {
        //let mut cmd1 = cmd!("echo "test1234" 1>{wrpipe}");
    }
}
