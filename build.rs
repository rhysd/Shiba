use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "Shiba")
            .set("FileDescription", "Shiba: Simple markdown previewer")
            .set("LegalCopyright", "Copyright (c) 2015 rhysd")
            .compile()?;
    }

    let profile = env::var("PROFILE").unwrap();
    if profile == "release" {
        let mut input = PathBuf::from("src");
        input.push("assets");
        input.push("bundle.min.js");
        println!("cargo:rerun-if-changed={}", input.display());

        let bundle = fs::read(input)?;
        let encoded = zstd::encode_all(bundle.as_slice(), 11)?;
        let mut output = PathBuf::from(env::var("OUT_DIR").unwrap());
        output.push("bundle.min.js.zstd");
        fs::write(output, encoded)?;
    }

    Ok(())
}
