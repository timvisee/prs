/// Transform binary into path on platforms that need it.
///
/// This transforms a binary name into a full path on Windows.
pub fn bin_path<S: AsRef<str>>(bin: S) -> String {
    #[cfg(not(windows))]
    {
        bin.as_ref().to_owned()
    }
    #[cfg(windows)]
    {
        match which::which(bin.as_ref()) {
            Ok(path) => path.to_str().unwrap().to_string(),
            Err(err) => panic!("failed to find binary '{}': {:?}", bin.as_ref(), err),
        }
    }
}
