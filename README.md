# defradb-rs

Prototype defradb impl in pure rust

## status

* [x] keyring impl complete 
* [x] keyring tests can interop with go keyring impl (see `/keyring/tests/crypto_tests.rs`)
* [x] corekv badger store impl is complete
* [x] corekv badger store tests full parity with go corekv (see `/corekv/tests/integration/*`, 1013 passing tests, includes all the test *multipliers* and so on)
