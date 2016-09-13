//! Error handling for the cpuprofiler thanks to error_chain!

use ProfilerState;
use std::io;
use std::ffi;
use std::str;

error_chain! {
    foreign_links {
        io::Error, Io;
        ffi::NulError, Nul;
        str::Utf8Error, Utf8;
    }

    errors {
        InternalError {
            description("Internal library error!")
            display("Internal library error!")
        }
        InvalidState(state: ProfilerState) {
            description("Operation is invalid for profiler state")
            display("Operation is invalid for profiler state: {}", state)
        }
    }
}
