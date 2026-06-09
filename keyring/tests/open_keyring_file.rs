mod tests {

    use keyring::{KeyringFile, KeyringT};

    // ref: defradb/keyring/file_test.go -> TestFileKeyring
    #[test]
    fn test_file_keyring() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let password = b"secret";

        let mut keyring = KeyringFile::open(dir.path(), password).expect("open keyring file");
        println!("Keyring file opened successfully: {:?}", keyring);

        keyring.set("peer_key", b"abc").expect("set peer_key");
        keyring.set("node_key", b"123").expect("set node_key");

        let peer_key = keyring.get("peer_key").expect("get peer_key");
        assert_eq!(peer_key, b"abc");

        let node_key = keyring.get("node_key").expect("get node_key");
        assert_eq!(node_key, b"123");

        keyring.delete("node_key").expect("delete node_key");
        let result = keyring.get("node_key");
        assert!(result.is_err(), "node_key should have been deleted");
    }
}
