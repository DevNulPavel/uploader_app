mod qr;

use std::{
    path::{
        // PathBuf,
        Path
    }
};

use reqwest::{
    Client
};
use tokio::{
    fs::{
        remove_file
    }
};
use slack_client_lib::{
    *
};
use self::{
    qr::{
        create_qr_data
    }
};

pub fn setup_logs() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(||{
        if std::env::var("RUST_LOG").is_err(){
            std::env::set_var("RUST_LOG", "debug");
        }
        env_logger::builder().is_test(true).init();
    })
}

fn build_client() -> SlackClient{
    // Slack api token
    let slack_api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");

    let client = SlackClient::new(Client::new(), slack_api_token);

    client
}


#[tokio::test]
async fn test_messages() {
    setup_logs();

    let client = build_client();

    client
        .send_message("Test message", SlackUserMessageTarget::new("U0JU3ACSJ"))
        .await
        .expect("Direct message failed");

    let formatted_text = format!("*Jenkins bot error:*```{}```", "TEST");
    let mut message = client
        .send_message(&formatted_text, SlackUserMessageTarget::new("U0JU3ACSJ"))
        .await
        .expect("Formatted direct message failed")
        .expect("Direct message - message object does not exist");


    tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        
    message
        .update_text("New text")
        .await
        .expect("Direct message update failed");

    let mut message = client
        .send_message("Test message", SlackChannelMessageTarget::new("#mur-test_node_upload"))
        .await
        .expect("Channel message failed")
        .expect("Channel message - message object does not exist");

    message
        .update_text("New text")
        .await
        .expect("Channel message update failed");

    client
        .send_message("Test message", SlackThreadMessageTarget::new(message.get_channel_id(), message.get_thread_id()))
        .await
        .expect("Thread message failed")
        .expect("Thread message object get failed");

    client
        .send_message("Test message", SlackEphemeralMessageTarget::new("#mur-test_node_upload", "U0JU3ACSJ"))
        .await
        .expect("Ephemeral message failed");

    // TODO: RESPONSE URL может фейлиться, не протестировано
}

#[tokio::test]
async fn test_image_upload() {
    setup_logs();

    let client = build_client();

    let image_data = create_qr_data("This is test text")
        .expect("Qr code create failed");

    assert_eq!(image_data.len() > 0, true);

    // Channel
    client
        .send_image(image_data.clone(), Some("Test commentary".to_owned()), SlackChannelImageTarget::new("#mur-test_node_upload"))
        .await
        .expect("Image send failed");


    // Thread
    let message = client
        .send_message("Test message", SlackChannelMessageTarget::new("#mur-test_node_upload"))
        .await
        .expect("Channel message failed")
        .expect("Channel message - message object does not exist");

    client
        .send_image(image_data.clone(), None, SlackThreadImageTarget::new(message.get_channel_id(), message.get_thread_id()))
        .await
        .expect("Image send failed");

    // Direct message
    client
        .send_image(image_data.clone(), None, SlackUserImageTarget::new("U0JU3ACSJ"))
        .await
        .expect("Image send failed");
}

#[tokio::test]
async fn test_find_user() {
    setup_logs();

    let client = build_client();

    let email_result = client
        .find_user_id_by_email("pershov@game-insight.com")
        .await
        .expect("Find user by email failed");

    assert_eq!(email_result, "U0JU3ACSJ");

    let name_result = client
        .find_user_id_by_name("Pavel Ershov", Option::None)
        .await
        .expect("Find user by name failed");
    assert_eq!(name_result, "U0JU3ACSJ");

    {
        let path = Path::new("test_cache.json");

        let cache = UsersJsonCache::new(path.to_owned())
            .await;
            
        let name_result = client
            .find_user_id_by_name("Pavel Ershov", Some(&cache))
            .await
            .expect("Find user by name failed");
        assert_eq!(name_result, "U0JU3ACSJ");

        remove_file(path).await.expect("Remove file failed");
    }

    {
        let path = Path::new("test_cache.sqlite");

        let cache = UsersSqliteCache::new(path.to_owned())
            .await
            .expect("SQLite cache create failed");

        let name_result = client
            .find_user_id_by_name("Pavel Ershov", Some(&cache))
            .await
            .expect("Find user by name failed");
        assert_eq!(name_result, "U0JU3ACSJ");

        remove_file(path).await.expect("Remove file failed");
    }
}