ENCRYPT_AUTH:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json.asc -e test_google_play_auth.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_island-tales_auth.json.asc -e test_google_play_island-tales_auth.json

DECRYPT_AUTH:
	rm -rf test_google_play_auth.json
	rm -rf test_google_play_island-tales_auth.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_auth.json -d test_google_play_auth.json.asc
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_play_island-tales_auth.json -d test_google_play_island-tales_auth.json.asc

TEST_LIB_ENTEGRATION:
	cargo test -- integration_test

TEST_REQUEST_BUILDER: 
	cargo test -- request_builder::tests::test_request_builder --exact --nocapture
