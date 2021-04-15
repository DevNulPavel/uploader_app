TEST_SSH_UPLOADING:
	cargo test -- uploaders::ssh::tests::test_ssh_uploader

BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

ENCRYPT_AUTH:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_my_auth.json.asc -e test_google_drive_my_auth.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_classic_auth.json.asc -e test_google_play_classic_auth.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json.asc -e test_google_play_auth.json


DECRYPT_AUTH:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_my_auth.json -d test_google_drive_my_auth.json.asc
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_classic_auth.json -d test_google_play_classic_auth.json.asc
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json -d test_google_play_auth.json.asc

BENCH:
	source test_environment.env && \
	cargo run --release --features=flame_it

TEST_ENV_PARAMETERS:
	source test_environment.env && \
	cargo test -- env_parameters::tests

EXAMPLE_APP_PARAMETERS:
	cargo run --bin example_cli_params -- --help

TEST_APP: BUILD
	bash test_uploading.sh

INSTALL_APP:
	cargo build --release
	mkdir -p ../uploader_app_bin
	yes | cp -rf target/release/uploader_app ../uploader_app_bin/uploader_app 