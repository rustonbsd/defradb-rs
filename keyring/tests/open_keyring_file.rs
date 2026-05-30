mod tests {
    use keyring::KeyringFile;

    #[test]
    fn test_file_keyring() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let password = b"secret";

        let keyring = KeyringFile::open(dir.path(), password).expect("failed to open keyring file");
        println!("Keyring file opened successfully: {:?}", keyring);
    }
}