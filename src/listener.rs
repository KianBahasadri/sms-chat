use std::{collections::HashMap, sync::{mpsc::Sender, Arc, Mutex}};

use axum::{extract::{Query, State}, routing::get, Router};
use log::{error, info};
use ngrok;
use reqwest;

use crate::TwilioConfig;

pub async fn setup_listener(twilio_creds: TwilioConfig, ngrok_authtoken: String, sender: Sender<(String, String)>) {
  let listener = get_ngrok_listener(ngrok_authtoken).await;
  let ngrok_url = ngrok::tunnel::UrlTunnel::url(&listener).to_owned();
  info!("Ngrok URL: {:?}", &ngrok_url);
  
  set_twilio_webhook(twilio_creds, &ngrok_url).await;
  info!("Set Ngrok endpoint as twilio webhook");
  let arc_sender = Arc::new(Mutex::new(sender));
  let axum_app: Router = Router::new().route("/", get(query_handler)).with_state(arc_sender);
  if let Err(e) = axum::Server::builder(listener).serve(axum_app.into_make_service()).await {
    error!("Listener error: {e}");
    panic!();
  }
}

async fn set_twilio_webhook(twilio_creds: TwilioConfig, ngrok_url: &str) {
  let client = reqwest::Client::new();
  
  let url = format!(
    "https://api.twilio.com/2010-04-01/Accounts/{}/IncomingPhoneNumbers/{}.json",
    twilio_creds.twilio_account_sid, twilio_creds.twilio_number_sid,
  );
  
  let params = [("SmsUrl", ngrok_url)];
  let response = client.post(&url).form(&params)
  .basic_auth(
    &twilio_creds.twilio_account_sid,
    Some(&twilio_creds.twilio_auth_token),
    ).send().await;

  match response {
    Ok(resp) => match resp.text().await {
      Ok(_) => info!("Ngrok webhook set up"),
      Err(e) => {
        error!("Failed to read response text: {}", e);
        panic!();
      }
    },
    Err(e) => {
      error!("Request error: {}", e);
      panic!();
    }
  }
}

async fn get_ngrok_listener(ngrok_authtoken: String) -> ngrok::tunnel::HttpTunnel {
  let session = match ngrok::Session::builder().authtoken(ngrok_authtoken).connect().await {
    Ok(s) => s,
    Err(e) => {
      error!("Creating Ngrok session, error: {}", e);
      panic!();
    }
  };
  info!("Opened Ngrok session.");

  let listener = match ngrok::config::TunnelBuilder::listen(&session.http_endpoint()).await {
    Ok(l) => l,
    Err(e) => {
      error!("Failed to listen: {e}");
      panic!();
    }
  };
  info!("Started Ngrok listener");

  listener
}

async fn query_handler(State(arc_sender): State<Arc<Mutex<Sender<(String, String)>>>>, Query(params): Query<HashMap<String, String>>) {
  let text = params["Body"].clone();
  let number = params["From"].clone();
  info!("Recieved text: {:?}: {:?}", &number, &text);
  info!("Waiting for Mutex lock on App:");
  let sender = arc_sender.lock().unwrap();
  sender.send((text, number)).unwrap();
}
