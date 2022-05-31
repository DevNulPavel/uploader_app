# ENCRYPT_AUTH:
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json.asc -e test_google_play_auth.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_island-tales_auth.json.asc -e test_google_play_island-tales_auth.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o google_play_beta_credentials.json.asc -e google_play_beta_credentials.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o google_play_classic_credentials.json.asc -e google_play_classic_credentials.json

# DECRYPT_AUTH:
# 	rm -rf test_google_play_auth.json
# 	rm -rf test_google_play_island-tales_auth.json
# 	rm -rf google_play_beta_credentials.json
# 	rm -rf google_play_classic_credentials.json
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json -d test_google_play_auth.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_island-tales_auth.json -d test_google_play_island-tales_auth.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o google_play_beta_credentials.json -d google_play_beta_credentials.json.asc
# 	gpg -a -r 0x0BD10E4E6E578FB6 -o google_play_classic_credentials.json -d google_play_classic_credentials.json.asc

TEST_LIB_ENTEGRATION:
	cargo test -- integration_test

TEST_REQUEST_BUILDER: 
	cargo test -- request_builder::tests::test_request_builder --exact --nocapture
