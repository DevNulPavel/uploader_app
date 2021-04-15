#! /usr/bin/env bash

# Import test environment
# eval "$(gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc)"
# gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc -o test_environment.env
source test_environment.env

# Print environment
# env | grep -i "GOOGLE_DRIVE"

# Rust environment setup
# export RUST_LOG=uploader_app=trace
export RUST_BACKTRACE=1
export RUST_LOG=uploader_app=trace,app_center_client=trace,google_drive_client=trace,reqwest=trace

# App center
# UPLOAD_PARAMS=()
# UPLOAD_PARAMS+=(--app_center_input_file "/Users/devnul/Downloads/app-release.apk")
# UPLOAD_PARAMS+=(--app_center_build_description "Test description")
# UPLOAD_PARAMS+=(--app_center_distribution_groups 'Paradise Island 2 Team','Collaborators')
# UPLOAD_PARAMS+=(--app_center_build_version '12.9.4')
# UPLOAD_PARAMS+=(--app_center_build_code '376')
# target/debug/uploader_app "${UPLOAD_PARAMS[@]}"

# Google drive
# --google_drive_target_domain ""
# target/debug/uploader_app \
#     --google_drive_files "/Users/devnul/Downloads/Mimestream_0.9.2.dmg" \
#     --google_drive_target_folder_id "1YtSfyiMp-MxF5AVWq_VnJxGtAwiMghBF" \
#     --google_drive_target_subfolder_name "NewFolderTest" \
#     --google_drive_target_owner_email "devnulpavel@gmail.com"

# Google play
UPLOAD_PARAMS=()
UPLOAD_PARAMS+=(--google_play_upload_file '/Users/devnul/Downloads/IslandTales-universal-gplay-12.3.0-42-14042021_1827-10c4f698/IslandTales-universal-gplay-12.3.0-42-14042021_1827-10c4f698.aab')
UPLOAD_PARAMS+=(--google_play_target_track 'internal')
UPLOAD_PARAMS+=(--google_play_package_name 'com.gameinsight.gplay.islandtales')
target/debug/uploader_app "${UPLOAD_PARAMS[@]}"