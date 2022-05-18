// commander.allowUnknownOption();

//////////////////////////////////////////////////////////////////////

params_data_type!(
    AmazonParams{
        Req{
            file_path : "amazon_input_file" : "Amazon uploading file"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    AppCenterParams{
        Req{
            input_file : "app_center_input_file" : "App center input file"
        }
        Opt{
            symbols_file: "app_center_symbols_file" : "App center symbols file",
            build_description: "app_center_build_description": "App center build description",
            build_version: "app_center_build_version": "App center build version",
            build_code: "app_center_build_code": "App center build code"
        }
        MultOpt{
            distribution_groups: "app_center_distribution_groups": "App center distribution groups"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    GoogleDriveParams{
        Req{
            target_folder_id : "google_drive_target_folder_id" : "Google drive folder ID"
        }
        Opt{
            target_subfolder_name : "google_drive_target_subfolder_name" : "Google drive subfolder name",
            target_owner_email : "google_drive_target_owner_email" : "Google drive folder owner email",
            target_domain: "google_drive_target_domain" : "Google drive shared domain"
        }
        Mult {
            files : "google_drive_files" : "Comma separated files list"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    GooglePlayParams{
        Req{
            file_path : "google_play_upload_file" : "File path for google play uploading",
            package_name: "google_play_package_name" : "Package name"
        }
        Opt{
            target_track : "google_play_target_track" : "Target track for google play build"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    IOSParams{
        Req{
            ipa_file_path : "ios_app_store_ipa" : "Ipa file for iOS App store"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    WindowsStoreParams{
        Req{
            zip_file_path : "windows_zip_file_path" : "ZIP file with .appx or .appxupload inside",
            test_flight_name : "windows_test_flight_name": "Test flight name in admin console"
        }
        Mult {
            test_groups : "windows_flight_groups" : "Microsoft flight groups for tests"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    SSHParams{
        Req{
            target_dir : "ssh_target_server_dir" : "Target server directory for files"
        }
        Mult {
            files : "ssh_upload_files" : "Comma separated input files"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    SlackParams{
        Opt{
            channel : "slack_upload_channel" : "Slack upload files channel",
            user: "slack_user" : "Slack user name for direct messages",
            email: "slack_user_email" : "Slack user email for direct messages",
            text: "slack_text" : "Slack message text",
            qr_commentary: "slack_qr_commentary" : "Slack QR code commentary",
            qr_text: "slack_qr_text" : "Slack direct QR code content"
        }
        Mult {
            files : "slack_upload_files" : "Comma separated input files"
        }
    }
);
