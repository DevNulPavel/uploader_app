ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	rm -rf test_environment.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

TEST_LIB_ENTEGRATION:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- integration_test

TEST_TOKEN_MOCK: 
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- token::tests::mock_test_azure_token --exact --nocapture

TEST_REQUEST_BUILDER: 
	cargo test -- request_builder::tests::test_request_builder --exact --nocapture
