use std::collections::HashMap;
use std::fs;
use std::env;
use std::io;
use reqwest;
use axum::{routing::get, Router, extract::Query};
use ngrok::prelude::*;
use tokio::spawn;

#[derive(Clone)]
struct TwilioConfig {
  twilio_account_sid: String,
  twilio_auth_token: String,
  twilio_number: String,
  to_number: String,
  twilio_number_sid: String,
}

#[tokio::main]
async fn main() {
  let twilio_creds: TwilioConfig = load_twilio_configs();
  let client = reqwest::Client::new();
  let ngrok_authtoken = get_ngrok_authtoken();
  
  let twilio_creds_bg = twilio_creds.clone();
  let ngrok_authtoken_bg = ngrok_authtoken.clone();
  let client_bg = client.clone();

  spawn(setup_listener(twilio_creds_bg, ngrok_authtoken_bg, client_bg));
  chat_loop(&client, &twilio_creds).await;
}

async fn setup_listener(
  twilio_creds: TwilioConfig,
  ngrok_authtoken: String,
  client: reqwest::Client,
) {
  // build our application with a route
  let app: Router = Router::new().route("/", get(query));

  // Attempt to build and connect an ngrok session
  let session = match ngrok::Session::builder()
      .authtoken(ngrok_authtoken)
      .connect()
      .await
  {
      Ok(s) => s,
      Err(e) => {
          eprintln!("Failed to connect ngrok session: {e}");
          return; // or handle differently
      }
  };

  // Attempt to listen on an HTTP endpoint
  let listener = match session.http_endpoint().listen().await {
      Ok(l) => l,
      Err(e) => {
          eprintln!("Failed to listen: {e}");
          return;
      }
  };

  let app_url = listener.url().to_owned();
  println!("App URL: {:?}", app_url);

  // Construct the URL
  let url = format!(
    "https://api.twilio.com/2010-04-01/Accounts/{}/IncomingPhoneNumbers/{}.json",    
    twilio_creds.twilio_account_sid, 
    twilio_creds.twilio_number_sid,
);

// Prepare the form data
let params = [("SmsUrl", app_url)];

println!("{}", twilio_creds.twilio_number_sid);
// Send the POST request
let response = client
    .post(&url)
    .form(&params)
    .basic_auth(
        &twilio_creds.twilio_account_sid, 
        Some(&twilio_creds.twilio_auth_token),
    )
    .send()
    .await;

match response {
    Ok(resp) => {
        match resp.text().await {
            Ok(_) => println!("Ngrok webhook set up"),
            Err(e) => eprintln!("Failed to read response text: {}", e),
        }
    }
    Err(e) => {
        eprintln!("Request error: {}", e);
    }
}

  // Serve Axum
  if let Err(e) = axum::Server::builder(listener)
      .serve(app.into_make_service())
      .await
  {
      eprintln!("Server error: {e}");
      return;
  }



}

async fn query(Query(params): Query<HashMap<String, String>>) {
  println!("{:?}: {:?}", params["From"], params["Body"]);
}

async fn chat_loop(client: &reqwest::Client, twilio: &TwilioConfig) {
  println!("Send a message!");
  loop {
    let mut usr_msg = String::new();
    io::stdin()
      .read_line(&mut usr_msg)
      .expect("Failed to read input");
    send_msg(client, twilio, &usr_msg.trim()).await;
    println!("{} (you): {}", twilio.twilio_number, usr_msg)
  }
}

fn get_ngrok_authtoken() -> String {
  let path_to_conf: String = format!("{}/.config/sms-chat/ngrok.conf", env::home_dir().unwrap().display());
  let conf_file: String = fs::read_to_string(&path_to_conf)
    .expect(&format!("Could not read {}, sowwy", &path_to_conf));
  let lines: Vec<&str> = conf_file.split_ascii_whitespace().collect();
  lines[0].to_string()
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
  let twilio_number_sid: String = lines[3].to_string();

  TwilioConfig {
    twilio_account_sid,
    twilio_auth_token,
    twilio_number,
    to_number,
    twilio_number_sid,
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
                Ok(_) => (), //println!("Success: {}", text),
                Err(e) => println!("Failed to read response text: {}", e),
            }
        }
        Err(e) => {
            println!("Request error: {}", e);
        }
    }
}

