# ENCRYPT_AUTH:
# 	rm -rf test_google_drive_my_auth.json.asc
# 	rm -rf test_google_drive_gi_auth_new.json.asc
# 	rm -rf test_google_drive_gi_auth_old.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_my_auth.json.asc -e test_google_drive_my_auth.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_gi_auth_new.json.asc -e test_google_drive_gi_auth_new.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_gi_auth_old.json.asc -e test_google_drive_gi_auth_old.json

# DECRYPT_AUTH:
# 	rm -rf test_google_drive_my_auth.json
# 	rm -rf test_google_drive_gi_auth_new.json
# 	rm -rf test_google_drive_gi_auth_old.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_my_auth.json -d test_google_drive_my_auth.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_gi_auth_new.json -d test_google_drive_gi_auth_new.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_gi_auth_old.json -d test_google_drive_gi_auth_old.json.asc

TEST_LIB_ENTEGRATION: DECRYPT_AUTH
	cargo test -- integration_test
