//! Cpu Profiler
//!
//! This crate provides safe bindings to google's cpuprofiler library.
//! This allows us to use statistical sampling to profile sections of code
//! and consume the output in a number of ways using [pprof](https://github.com/google/pprof).
//! 
//! # Installation
//!
//! In order to use this library you will need to install [gperftools](https://github.com/gperftools/gperftools). There are instructions
//! in their repository but it's roughly the following:
//!
//! 1. Download package from [releases](https://github.com/gperftools/gperftools/releases)
//! 2. Run `./configure`
//! 3. Run `make install`
//!
//! There may be some other dependencies for your system - these are explained well in their
//! [INSTALL](https://github.com/gperftools/gperftools/blob/master/INSTALL) document.
//! For example [libunwind](http://download.savannah.gnu.org/releases/libunwind/) (> 0.99.0) is required for 64 bit systems.
//!
//! # Usage
//!
//! ```
//! use cpuprofiler::PROFILER;
//!
//! PROFILER.lock().unwrap().start("./my-prof.profile");
//! // Code you want to sample goes here!
//! PROFILER.lock().unwrap().stop();
//! ```
//!
//! The profiler is accessed via the static `PROFILER: Mutex<Profiler>`.
//! We limit access this way to ensure that only one profiler is running at a time -
//! this is a limitation of the cpuprofiler library.

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

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
    /// Static reference to the PROFILER
    ///
    /// The cpuprofiler library only supports one active profiler.
    /// Because of this we must use static access and wrap in a `Mutex`.
    #[derive(Debug)]
    pub static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler {
        state: ProfilerState::NotActive,
    });
}

#[link(name = "profiler")]
#[allow(non_snake_case)]
extern "C" {
    fn ProfilerStart(fname: *const c_char) -> i32;

    fn ProfilerStop();
}

/// The state of the profiler
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfilerState {
    /// When the profiler is active
    Active,
    /// When the profiler is inactive
    NotActive,
}

impl fmt::Display for ProfilerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ProfilerState::Active => write!(f, "Active"),
            ProfilerState::NotActive => write!(f, "NotActive"),
        }
    }
}

/// The `Profiler`
///
/// The `Profiler` gives access to the _cpuprofiler_ library.
/// By storing the state of the profiler and limiting access
/// we make the FFI safer.
#[derive(Debug)]
pub struct Profiler {
    state: ProfilerState,
}

impl Profiler {

    /// Returns the profiler state
    ///
    /// # Examples
    ///
    /// ```
    /// use cpuprofiler::PROFILER;
    ///
    /// println!("{}", PROFILER.lock().unwrap().state());
    /// ```
    pub fn state(&self) -> ProfilerState {
        self.state
    }

    /// Start the profiler
    ///
    /// Will begin sampling once this function has been called
    /// and will not stop until the `stop` function has been called.
    ///
    /// This function takes as an argument a filename. The filename must be
    /// both valid Utf8 and a valid `CString`.
    ///
    /// # Failures
    ///
    /// - The profiler is currently `Active`.
    /// - `fname` is not a valid `CString`.
    /// - `fname` is not valid Utf8.
    /// - `fname` is not a file.
    /// - The user does not have write access to the file.
    /// - An internal failure from the cpuprofiler library.
    pub fn start<T: Into<Vec<u8>>>(&mut self, fname: T) -> Result<(), Error> {
        if self.state == ProfilerState::NotActive {
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
                        self.state = ProfilerState::Active;
                        Ok(())
                    }
                }
            }
        } else {
            Err(ErrorKind::InvalidState(self.state).into())
        }
    }

    /// Stop the profiler.
    ///
    /// This will stop the profiler if it `Active` and return
    /// an error otherwise.
    ///
    /// # Failures
    ///
    /// - The profiler is `NotActive`.
    pub fn stop(&mut self) -> Result<(), Error> {
        if self.state == ProfilerState::Active {
            unsafe {
                ProfilerStop();
            }
            self.state = ProfilerState::NotActive;
            Ok(())
        } else {
            Err(ErrorKind::InvalidState(self.state).into())
        }
    }
}
