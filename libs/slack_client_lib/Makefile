1_GENERATE_GPG_KEY:
	gpg --full-generate-key
	open ~/.gnupg/

2_ENCRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -e: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

3_DECRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -d: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

4_EXPORT_CHAIN:
	# -a: ASCII
	# -r: Key fingerpring
	# -d: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

5_TEST_ENV_SETUP:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc)

TEST: 
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test

TEST_FIND_USER: 
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- tests::test_find_user

TEST_USERS_CACHE: 
	export RUST_LOG=trace && \
	export RUST_BACKTRACE=1 && \
	cargo test -- users_cache

TEST_USERS_JSON_CACHE: 
	cargo test -- test_json_cache

TEST_USERS_SQLITE_CACHE: 
	export RUST_LOG=trace && \
	export RUST_BACKTRACE=1 && \
	cargo test -- test_sqlite_cache
