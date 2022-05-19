#! /usr/bin/env bash

# Import test environment
# eval "$(gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc)"
# gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc -o test_environment.env
source env/test_environment.env

# Print environment
# env | grep -i "GOOGLE_DRIVE"

# Rust environment setup
export RUST_BACKTRACE=1
# export RUST_LOG=debug
# export RUST_LOG=uploader_app=debug,google_drive_client=debug
export RUST_LOG=uploader_app=debug,microsoft_azure_client=debug
# export RUST_LOG=uploader_app=trace,app_center_client=trace,google_drive_client=trace,reqwest=trace

# App center
# UPLOAD_PARAMS=()
# UPLOAD_PARAMS+=(--app_center_input_file "/Users/devnul/Downloads/Island2_12.12.5.0_x86.appxbundle")
# UPLOAD_PARAMS+=(--app_center_build_description "Test description")
# UPLOAD_PARAMS+=(--app_center_distribution_groups "ParadiseIsland2Team","Collaborators")
# UPLOAD_PARAMS+=(--app_center_build_version '12.9.4')
# UPLOAD_PARAMS+=(--app_center_build_code '376')
# target/debug/uploader_app "${UPLOAD_PARAMS[@]}"

# Google drive
# UPLOAD_PARAMS=()
# UPLOAD_PARAMS+=(--google_drive_files "/Users/devnul/Downloads/airshipper-macos.tar.gz")
# UPLOAD_PARAMS+=(--google_drive_target_folder_id "1YtSfyiMp-MxF5AVWq_VnJxGtAwiMghBF")
# UPLOAD_PARAMS+=(--google_drive_target_subfolder_name "NewFolderTest")
# UPLOAD_PARAMS+=(--google_drive_target_owner_email 'devnulpavel@gmail.com')
# target/debug/uploader_app "${UPLOAD_PARAMS[@]}"

# Google play
# UPLOAD_PARAMS=()
# UPLOAD_PARAMS+=(--google_play_upload_file '/Users/devnul/Downloads/Island2-universal-gplay-v12.12.6-b391-28042021_1742-11ad1b3e/Island2-universal-gplay-v12.12.6-b391-28042021_1742-11ad1b3e.aab')
# UPLOAD_PARAMS+=(--google_play_target_track 'internal')
# UPLOAD_PARAMS+=(--google_play_package_name 'com.gameinsight.gplay.island2')
# target/debug/uploader_app "${UPLOAD_PARAMS[@]}"

# SSH
# UPLOAD_PARAMS=()
# UPLOAD_PARAMS+=(--ssh_target_server_dir '~/test_folder')
# UPLOAD_PARAMS+=(--ssh_upload_files '/Users/devnul/projects/uploader_app_src/Makefile')
# target/debug/uploader_app "${UPLOAD_PARAMS[@]}"

# Microsoft store
UPLOAD_PARAMS=()
UPLOAD_PARAMS+=(--windows_app_id '9PBPBN166FXW')
UPLOAD_PARAMS+=(--windows_production_submission_name 'Production build')
UPLOAD_PARAMS+=(--windows_production_zip_file_path '/Users/devnul/Downloads/MHouseXGen_5.161.0.0_Win32_TEST_UPLOAD.appxupload.zip')
# UPLOAD_PARAMS+=(--windows_test_flight_name 'Flight name test')
# UPLOAD_PARAMS+=(--windows_test_flight_groups '1152921504607280735')
# UPLOAD_PARAMS+=(--windows_test_flight_zip_file_path '/Users/devnul/Downloads/MHouseXGen_5.161.0.0_Win32_TEST_UPLOAD.appxupload.zip')
target/debug/uploader_app "${UPLOAD_PARAMS[@]}"
