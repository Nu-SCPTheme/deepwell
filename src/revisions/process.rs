/*
 * revisions/process.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith
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
use std::ffi::OsStr;
use std::fmt::Write;
use std::io::Read;
use std::time::Duration;
use subprocess::{ExitStatus, Popen, PopenConfig, Redirection};

const TIMEOUT: Duration = Duration::from_millis(200);

macro_rules! mut_borrow {
    ($option:expr) => (
        $option.as_mut().unwrap()
    )
}

pub fn spawn(arguments: &[&OsStr]) -> Result<()> {
    spawn_inner(arguments, false).map(|_| ())
}

pub fn spawn_output(arguments: &[&OsStr]) -> Result<Box<[u8]>> {
    spawn_inner(arguments, true).map(|out| out.unwrap())
}

fn spawn_inner(arguments: &[&OsStr], output: bool) -> Result<Option<Box<[u8]>>> {
    let config = PopenConfig {
        stdin: Redirection::Pipe,
        stdout: Redirection::Pipe,
        stderr: Redirection::Pipe,
        ..PopenConfig::default()
    };

    let mut popen = Popen::create(arguments, config)?;
    match popen.wait_timeout(TIMEOUT)? {
        Some(status) if status.success() => {
            if output {
                let stdout = mut_borrow!(popen.stdout);
                let mut buffer = Vec::new();
                stdout.read_to_end(&mut buffer)?;
                let bytes = buffer.into_boxed_slice();

                Ok(Some(bytes))
            } else {
                Ok(None)
            }
        }
        Some(status) => {
            let mut buffer = String::new();
            let stderr = mut_borrow!(popen.stderr);
            let written = stderr.read_to_string(&mut buffer)?;

            if written != 0 {
                buffer.insert_str(0, "command failed: ");
            }

            match status {
                ExitStatus::Exited(code) => write!(&mut buffer, "({})", code).unwrap(),
                ExitStatus::Signaled(code) => write!(&mut buffer, "(killed by signal {})", code).unwrap(),
                _ => (),
            }

            Err(Error::CommandFailed(buffer))
        }
        None => {
            popen.terminate()?;
            let timeout = Duration::from_millis(100);
            let result = popen.wait_timeout(timeout)?;
            if result.is_none() {
                popen.kill()?;
            }

            Err(Error::CommandFailed(format!("command timed out ({} ms)", TIMEOUT.as_millis())))
        }
    }
}
