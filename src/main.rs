use std::fs;
use std::env;
use std::io;
use std::io::Read;
use reqwest;

struct TwilioConfig {
  twilio_account_sid: String,
  twilio_auth_token: String,
  twilio_number: String,
  to_number: String,
}

#[tokio::main]
async fn main() {
  let creds: TwilioConfig = load_twilio_configs();

  let client = reqwest::Client::new();
  chat_loop(&client, &creds).await;

}

async fn chat_loop(client: &reqwest::Client, twilio: &TwilioConfig) {
  loop {
    let mut usr_msg = String::new();
    io::stdin()
      .read_line(&mut usr_msg)
      .expect("Failed to read input");
    send_msg(client, twilio, &usr_msg.trim()).await;
    println!("{} (you): {}", twilio.twilio_number, usr_msg)
  }
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
    twilio_account_sid,
    twilio_auth_token,
    twilio_number,
    to_number,
  }
}

async fn send_msg(client: &reqwest::Client, twilio: &TwilioConfig, msg: &str) {
    // Construct the URL with the Account SID
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        twilio.twilio_account_sid
    );

    // Prepare the form data
    let params = [
        ("Body", msg),
        ("From", &twilio.twilio_number),
        ("To", &twilio.to_number),
    ];

    // Send the POST request
    let response = client
        .post(&url)
        .form(&params)
        .basic_auth(&twilio.twilio_account_sid, Some(&twilio.twilio_auth_token))
        .send()
        .await;

    // Handle the response
    match response {
        Ok(resp) => {
            // Await the text asynchronously
            match resp.text().await {
                Ok(text) => (), //println!("Success: {}", text),
                Err(e) => println!("Failed to read response text: {}", e),
            }
        }
        Err(e) => {
            println!("Request error: {}", e);
        }
    }
}

