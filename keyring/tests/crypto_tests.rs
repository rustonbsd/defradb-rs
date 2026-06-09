mod tests {
    use keyring::crypto::CryptoError;

    #[test]
    fn test_crypto_roundtrip() -> Result<(), CryptoError> {
        let password = b"secret";
        let plain_0 = b"hello world";

        let jwe = keyring::crypto::encrypt(plain_0, password)?;
        let plain_1 = keyring::crypto::decrypt(&jwe, password)?;

        assert_eq!(plain_0, plain_1.as_slice());
        Ok(())
    }

    // ref: defradb/keyring/rs_test.go -> TestRsKeyring_DecryptJWE
    // cargo test --package keyring --test crypto_tests -- tests::test_go_decrypt_jwe --nocapture
    #[test]
    fn test_go_decrypt_jwe() -> Result<(), CryptoError> {
        let password = b"secret";
        let plain_0 = "payload";
        let jwe = b"eyJhbGciOiJQQkVTMi1IUzUxMitBMjU2S1ciLCJlbmMiOiJBMjU2R0NNIiwicDJjIjoxMDAwMCwicDJzIjoiYTdRVG00SW4yMVdUeFQ4aW5EX0JmdDFnM1I0ODB1ZFY2UmdEMWswcVFOayJ9.0LZ7tea27lxbAMveWFjhvsE17hoW-xWJ_Gr480LcnmNI7L75b0ZkSw.tAxVzwHBIuk1-_HQ.4-Qu19yNHg.uLRdcjjvijlI_fOfbPSMyw";

        let plain_1 = keyring::crypto::decrypt(jwe, password)?;
        println!(
            "RsKeyring plain={} password={} jwe={}",
            std::str::from_utf8(&plain_1).expect("valid utf-8"),
            std::str::from_utf8(password).expect("valid utf-8"),
            std::str::from_utf8(jwe).expect("valid utf-8")
        );
        assert_eq!(plain_0.as_bytes(), plain_1.as_slice());
        Ok(())
    }

    // ref: defradb/keyring/rs_test.go -> TestRsKeyring_CreateJWE
    // cargo test --package keyring --test crypto_tests -- tests::test_go_create_jwe --nocapture
    #[test]
    fn test_go_create_jwe() -> Result<(), CryptoError> {
        let password = b"secret";
        let plain_0 = b"payload";
        let jwe = keyring::crypto::encrypt(plain_0, password)?;
        println!(
            "RsKeyring plain={} password={} jwe={}",
            std::str::from_utf8(plain_0).expect("valid utf-8"),
            std::str::from_utf8(password).expect("valid utf-8"),
            std::str::from_utf8(&jwe).expect("valid utf-8")
        );
        Ok(())
    }

    #[test]
    fn test_go_decrypt_jwe_wrong_password() -> Result<(), CryptoError> {
        let password = b"wrong password";
        let jwe = b"eyJhbGciOiJQQkVTMi1IUzUxMitBMjU2S1ciLCJlbmMiOiJBMjU2R0NNIiwicDJjIjoxMDAwMCwicDJzIjoiYTdRVG00SW4yMVdUeFQ4aW5EX0JmdDFnM1I0ODB1ZFY2UmdEMWswcVFOayJ9.0LZ7tea27lxbAMveWFjhvsE17hoW-xWJ_Gr480LcnmNI7L75b0ZkSw.tAxVzwHBIuk1-_HQ.4-Qu19yNHg.uLRdcjjvijlI_fOfbPSMyw";

        let plain_1 = keyring::crypto::decrypt(jwe, password);
        assert!(plain_1.is_err());
        Ok(())
    }
}
