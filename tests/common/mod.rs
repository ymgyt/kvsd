pub fn temp_dir() -> tempdir::TempDir {
    tempdir::TempDir::new("kvsd").unwrap()
}
