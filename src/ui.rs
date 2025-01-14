use log::info;
use std::io;

use crate::TwilioConfig;

pub fn get_number_to() -> String {
  println!("Please enter a phone number, include the country code, eg. +1647xxxXXXX");
  let mut to_number = String::new();
  io::stdin().read_line(&mut to_number).unwrap();
  to_number.trim().to_owned()
}

pub async fn chat_loop(twilio: &TwilioConfig, to_number: &str) {
  let client = reqwest::Client::new();
  println!("Send a message!");
  loop {
    let mut usr_msg = String::new();
    io::stdin()
      .read_line(&mut usr_msg)
      .expect("Failed to read input");
    send_msg(&client, twilio, &usr_msg.trim(), to_number).await;
    println!("\x1b[A{} (you): {}", twilio.twilio_number, usr_msg)
  }
}

async fn send_msg(client: &reqwest::Client, twilio: &TwilioConfig, msg: &str, to_number: &str) {
  info!("send_msg called with: {}", msg);

  let url = format!(
    "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
    twilio.twilio_account_sid
  );

  let params = [
    ("Body", msg),
    ("From", &twilio.twilio_number),
    ("To", to_number),
  ];

  let response = client
    .post(&url)
    .form(&params)
    .basic_auth(&twilio.twilio_account_sid, Some(&twilio.twilio_auth_token))
    .send()
    .await;

  match response {
    Ok(resp) => {
      match resp.text().await {
        Ok(text) => info!("Message sent, twilio response: {}", text),
        Err(e) => println!("Failed to read response text: {}", e),
      }
    }
    Err(e) => {
      println!("Request error: {}", e);
    }
  }
}
