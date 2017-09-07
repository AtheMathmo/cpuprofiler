//! Error handling for the cpuprofiler thanks to error_chain!

use ProfilerState;
use std::io;
use std::ffi;
use std::str;

error_chain! {
    foreign_links {
        Io(io::Error);
        Nul(ffi::NulError);
        Utf8(str::Utf8Error);
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
