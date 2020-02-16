/*
 * revision/process.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019-2020 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use crate::{Error, Result};
use async_std::future::timeout;
use futures::future::Future;
use futures::task::{Context, Poll};
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::io::Read;
use std::pin::Pin;
use std::time::Duration;
use subprocess::{ExitStatus, Popen, PopenConfig, Redirection};

const TIMEOUT: Duration = Duration::from_millis(1800);

macro_rules! mut_borrow {
    ($option:expr) => {
        $option.as_mut().unwrap()
    };
}

/// Runs a process to completion, returning `Err` if it fails.
pub async fn spawn(repo: OsString, arguments: &[&OsStr]) -> Result<()> {
    debug!(
        "Running process: (in {:?}) {:?} (no capture)",
        repo, arguments,
    );

    spawn_inner(repo, arguments, false).await.map(|_| ())
}

/// Runs a process to completion, returning its `stdout`, or `Err` if it fails.
pub async fn spawn_output(repo: OsString, arguments: &[&OsStr]) -> Result<Box<[u8]>> {
    debug!(
        "Running process: (in {:?}) {:?} (capturing stdout)",
        repo, arguments,
    );

    spawn_inner(repo, arguments, true)
        .await
        .map(|out| out.unwrap())
}

async fn spawn_inner(
    repo: OsString,
    arguments: &[&OsStr],
    output: bool,
) -> Result<Option<Box<[u8]>>> {
    let config = PopenConfig {
        stdin: Redirection::Pipe,
        stdout: Redirection::Pipe,
        stderr: Redirection::Pipe,
        cwd: Some(repo),
        ..PopenConfig::default()
    };

    let mut popen = match Popen::create(arguments, config) {
        Ok(popen) => popen,
        Err(error) => {
            warn!("Failed to created subprocess: {}", error);

            return Err(Error::Subprocess(error));
        }
    };

    trace!(
        "Created {:?}, waiting {} ms for completion",
        popen,
        TIMEOUT.as_millis(),
    );

    macro_rules! await_exit {
        ($popen:expr, $timeout:expr) => {
            timeout($timeout, PopenAsync::from(&mut $popen)).await
        };
    }

    match await_exit!(popen, TIMEOUT) {
        Ok(status) if status.success() => {
            trace!("Command succeeded, gathering stdout");

            if output {
                let stdout = mut_borrow!(popen.stdout);
                let mut buffer = Vec::new();
                stdout.read_to_end(&mut buffer)?;
                trace!("Gathered {} bytes of stdout", buffer.len());
                let bytes = buffer.into_boxed_slice();

                Ok(Some(bytes))
            } else {
                Ok(None)
            }
        }
        Ok(status) => {
            trace!("Command failed, status {:?}", status);

            let mut buffer = String::new();
            for argument in &arguments[..2] {
                write!(&mut buffer, "{} ", argument.to_string_lossy()).unwrap();
            }

            buffer.push_str("command failed: ");

            let stderr = mut_borrow!(popen.stderr);
            stderr.read_to_string(&mut buffer)?;

            match status {
                ExitStatus::Exited(code) => {
                    warn!("Process exited with non-zero status code {}", code);
                    write!(&mut buffer, "(exit status {})", code).unwrap();
                }
                ExitStatus::Signaled(code) => {
                    warn!("Process was killed by signal {}", code);
                    write!(&mut buffer, "(killed by signal {})", code).unwrap();
                }
                _ => {
                    warn!("Process was killed by unknown source ({:?})", status);
                    write!(&mut buffer, "(unknown cause)").unwrap();
                }
            }

            Err(Error::CommandFailed(buffer))
        }
        Err(_) => {
            warn!(
                "Process timed out after {} ms, terminating",
                TIMEOUT.as_millis(),
            );

            const KILL_TIMEOUT: Duration = Duration::from_millis(2000);

            if let Err(error) = popen.terminate() {
                warn!("Failed to terminate process: {}", error);
                return Err(Error::Io(error));
            }

            if let Err(_) = await_exit!(popen, KILL_TIMEOUT) {
                warn!("Process did not exit after termination, killing");
                popen.kill()?;
            }

            let message = format!("command timed out ({} ms)", TIMEOUT.as_millis());
            Err(Error::CommandFailed(message))
        }
    }
}

#[derive(Debug)]
struct PopenAsync<'p> {
    inner: &'p mut Popen,
}

impl<'p> From<&'p mut Popen> for PopenAsync<'p> {
    #[inline]
    fn from(inner: &'p mut Popen) -> Self {
        PopenAsync { inner }
    }
}

impl<'p> Future for PopenAsync<'p> {
    type Output = ExitStatus;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        let popen = &mut self.as_mut().inner;

        match popen.poll() {
            Some(status) => Poll::Ready(status),
            None => Poll::Pending,
        }
    }
}
