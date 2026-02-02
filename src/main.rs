// Do not pop up cmd.exe window when clicking .exe on Windows. This also disables any console outputs
// so logger output will not work.
// https://learn.microsoft.com/en-us/cpp/build/reference/subsystem-specify-subsystem
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions), not(feature = "__bench")),
    windows_subsystem = "windows"
)]

use anyhow::Result;
use env_logger::{Builder, Env};
use log::LevelFilter;
use shiba_preview::{Options, Parsed, run};
use std::env;
use std::process;

fn try_main() -> Result<()> {
    match Options::parse(env::args_os())? {
        Parsed::Options(options) => {
            let level = if options.debug { LevelFilter::Debug } else { LevelFilter::Info };
            let env = Env::new().filter("SHIBA_LOG").write_style("SHIBA_LOG_STYLE");
            Builder::new()
                .filter_level(level)
                .format_timestamp(None)
                .filter_module("html5ever", LevelFilter::Off)
                .parse_env(env)
                .init();
            run(options)
        }
        Parsed::Help(help) => {
            println!("{help}");
            Ok(())
        }
        Parsed::Version(version) => {
            println!("{version}");
            Ok(())
        }
    }
}

// Note: `fn main() -> Result<()>` doesn't work on Windows. When `Result::Err` is returned from `main`, the console
// is already detatched on Windows (in release mode). In this case the error message is not output to console and users
// cannot know what was the error.
fn main() {
    // When windows_subsystem is set to "windows", console outputs are detached from the process and no longer printed
    // on terminal. However we still want to see the logger output for debugging by running this binary from terminal.
    #[cfg(all(windows, not(debug_assertions)))]
    let _console = shiba_preview::WindowsConsole::attach();

    if let Err(err) = try_main() {
        eprintln!("Error: {err}");
        for cause in err.chain().skip(1) {
            eprintln!("  Caused by: {cause}");
        }
        process::exit(1);
    }
}
