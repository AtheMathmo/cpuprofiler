extern crate pkg_config;

fn main () {
    match pkg_config::Config::new().atleast_version("2.0").probe("libprofiler") {
        Ok(_) => (),
        Err(_) => {
            // Old gperftools do not come with a pkg-config file so just rely
            // on the linker's path.
            println!("cargo:rustc-link-lib=profiler");
        },
    };
}
