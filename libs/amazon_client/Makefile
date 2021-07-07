ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env --passphrase-file ~/.gpg_pass -d test_environment.env.asc

TEST_LIB_ENTEGRATION:
	source test_environment.env && \
	cargo test -- integration_test
