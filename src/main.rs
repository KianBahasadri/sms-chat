use std::fs;
use std::env;
use reqwest;

struct TwilioConfig {
  TWILIO_ACCOUNT_SID: String,
  TWILIO_AUTH_TOKEN: String,
  TWILIO_NUMBER: String,
  TO_NUMBER: String,
}

#[tokio::main]
async fn main() {
  let creds: TwilioConfig = load_twilio_configs();

  let client = reqwest::Client::new();
  send_msg(&client, &creds, "Hello").await;

}

fn load_twilio_configs() -> TwilioConfig {
  let args = env::args().collect::<Vec<String>>();
  if args.len() < 2 {
    println!("Please provide a phone number to text");
    panic!("implement smooth exit");
  }
  let to_number: String = args[1].clone();

  let path_to_conf: String = format!("{}/.config/sms-chat/twilio.conf", env::home_dir().unwrap().display());
  let conf_file: String = fs::read_to_string(&path_to_conf)
    .expect(&format!("Could not read {}, sowwy", &path_to_conf));
  let lines: Vec<&str> = conf_file.split_ascii_whitespace().collect();
  let twilio_account_sid: String = lines[0].to_string();
  let twilio_auth_token: String = lines[1].to_string();
  let twilio_number: String = lines[2].to_string();

  TwilioConfig {
    TWILIO_ACCOUNT_SID: twilio_account_sid,
    TWILIO_AUTH_TOKEN: twilio_auth_token,
    TWILIO_NUMBER: twilio_number,
    TO_NUMBER: to_number,
  }
}

async fn send_msg(client: &reqwest::Client, twilio: &TwilioConfig, msg: &str) {
    // Construct the URL with the Account SID
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        twilio.TWILIO_ACCOUNT_SID
    );

    // Prepare the form data
    let params = [
        ("Body", msg),
        ("From", &twilio.TWILIO_NUMBER),
        ("To", &twilio.TO_NUMBER),
    ];

    // Send the POST request
    let response = client
        .post(&url)
        .form(&params)
        .basic_auth(&twilio.TWILIO_ACCOUNT_SID, Some(&twilio.TWILIO_AUTH_TOKEN))
        .send()
        .await;

    // Handle the response
    match response {
        Ok(resp) => {
            // Await the text asynchronously
            match resp.text().await {
                Ok(text) => println!("Success: {}", text),
                Err(e) => println!("Failed to read response text: {}", e),
            }
        }
        Err(e) => {
            println!("Request error: {}", e);
        }
    }
}

