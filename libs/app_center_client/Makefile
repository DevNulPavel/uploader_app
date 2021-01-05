ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

TEST: 
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test

TEST_REQUEST_BUILDER: 
	cargo test -- request_builder

TEST_HELPERS: 
	cargo test -- helpers

TEST_LIB_ENTEGRATION: 
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test --release -- library_integration_test
