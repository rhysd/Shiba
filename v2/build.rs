fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "Shiba")
            .set("LegalCopyright", "Copyright (c) 2015 rhysd")
            .compile()?;
    }
    Ok(())
}
