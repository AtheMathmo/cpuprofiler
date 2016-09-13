//! Cpuprofiler wrapper

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

mod error;

use std::ffi::CString;
use std::fmt;
use std::fs;
use std::io;
use std::os::raw::c_char;

use error::{Error, ErrorKind};

use std::sync::Mutex;

lazy_static! {
    pub static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler {
        state: ProfilerState::Stopped,
    });
}

#[link(name = "profiler")]
#[allow(non_snake_case)]
extern "C" {
    fn ProfilerStart(fname: *const c_char) -> i32;

    fn ProfilerStop();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfilerState {
    Started,
    Stopped,
}

impl fmt::Display for ProfilerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ProfilerState::Started => write!(f, "Started"),
            ProfilerState::Stopped => write!(f, "Stopped"),
        }
    }
}

#[derive(Debug)]
pub struct Profiler {
    state: ProfilerState,
}

impl Profiler {
    pub fn start<T: Into<Vec<u8>>>(&mut self, fname: T) -> Result<(), Error> {
        if self.state == ProfilerState::Stopped {
            let c_fname = try!(CString::new(fname));

            let metadata = try!(fs::metadata(try!(c_fname.to_str())));

            if !metadata.is_file() {
                Err(io::Error::new(io::ErrorKind::NotFound, "Invalid file for profile").into())
            } else if metadata.permissions().readonly() {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "File is readonly").into())
            } else {
                unsafe {
                    let res = ProfilerStart(c_fname.as_ptr());
                    if res == 0 {
                        Err(ErrorKind::InternalError.into())
                    } else {
                        self.state = ProfilerState::Started;
                        Ok(())
                    }
                }
            }
        } else {
            Err(ErrorKind::InvalidState(self.state).into())
        }
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if self.state == ProfilerState::Started {
            unsafe {
                ProfilerStop();
            }
            self.state = ProfilerState::Stopped;
            Ok(())
        } else {
            Err(ErrorKind::InvalidState(self.state).into())
        }
    }
}
