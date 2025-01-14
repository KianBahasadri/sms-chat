use tokio::spawn;
use log::info;
use simple_logging;
use homedir;

// local modules
mod load_creds;
mod listener;
mod ui;

#[derive(Clone)]
struct TwilioConfig {
  twilio_account_sid: String,
  twilio_auth_token: String,
  twilio_number: String,
  twilio_number_sid: String,
}

#[tokio::main]
async fn main() {
  let mut conf_dir = homedir::my_home().unwrap().unwrap();
  conf_dir.push(".config/sms-chat/");
  let log_path = conf_dir.join("sms-chat.log");
  let conf_file = conf_dir.join("sms-chat.conf");
  simple_logging::log_to_file(log_path, log::LevelFilter::Info).expect("Error setting up logging");
  info!("Application Starting");

  let ngrok_authtoken = load_creds::get_ngrok_authtoken(&conf_file);
  info!("Recieved ngrok authentication token");

  let twilio_creds: TwilioConfig = load_creds::load_twilio_configs(&conf_file);
  info!("Recieved Twilio Creds");

  let to_number = &ui::get_number_to();
  
  spawn(listener::setup_listener(twilio_creds.clone(), ngrok_authtoken));
  info!("Spawned listener");

  info!("Starting chat loop");
  ui::chat_loop(&twilio_creds, to_number).await;
}