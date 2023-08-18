fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "Shiba")
            .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
            .compile()?;
    }
    Ok(())
}
