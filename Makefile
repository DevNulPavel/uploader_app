TEST_SSH_UPLOADING:
	cargo test -- uploaders::ssh::tests::test_ssh_uploader

BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

DECRYPT_CONFIGS:
	git-crypt unlock
	
BENCH:
	source env/test_environment.env && \
	cargo run --release --features=flame_it

TEST_ENV_PARAMETERS:
	source env/test_environment.env && \
	cargo test -- env_parameters::tests

EXAMPLE_APP_PARAMETERS:
	cargo run --bin example_cli_params -- --help

TEST_APP: BUILD
	bash test_uploading.sh

INSTALL_APP:
	cargo build --release
	mkdir -p ../uploader_app_bin
	yes | cp -rf target/release/uploader_app ../uploader_app_bin/uploader_app 