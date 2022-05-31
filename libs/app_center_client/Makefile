# ENCRYPT_TEST_ENV:
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

# DECRYPT_TEST_ENV:
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

TEST: 
	source env/test_environment.env && \
	cargo test

TEST_REQUEST_BUILDER: 
	cargo test -- request_builder

TEST_HELPERS: 
	cargo test -- helpers

TEST_LIB_ENTEGRATION: 
	source env/test_environment.env && \
	cargo test --release -- library_integration_test

RUN_EXAMPLE:
	source env/test_environment.env && \
	cargo run --example alloc_profile_test

PROFILE_EXAMPLE:
	source env/test_environment.env && \
	cargo instruments --template alloc --example alloc_profile_test