use std::path;
use configparser::ini::Ini;
use log::error;
use crate::TwilioConfig;

pub fn get_ngrok_authtoken(conf_file: &path::PathBuf) -> String {
  let mut config = Ini::new();
  
  if let Err(_) = config.load(conf_file) {
    error!("Opening {}", conf_file.display());
    panic!();
  }

  if let Some(token) = config.get("ngrok", "auth_token") {
    token
  } else {
      error!("Getting ngrok auth token from conf file");
      panic!();
  }
}
  
pub fn load_twilio_configs(conf_file: &path::PathBuf) -> TwilioConfig {
  let mut config = Ini::new();

  let twilio= match config.load(conf_file) {
    Ok(map) => map["twilio"].clone(),
    Err(e) => {
      error!("Opening {}, error: {}", conf_file.display(), e);
      panic!();
    }
  };

TwilioConfig {
  twilio_account_sid: twilio["account_sid"].as_ref().unwrap().to_owned(),
  twilio_auth_token: twilio["auth_token"].as_ref().unwrap().to_owned(),
  twilio_number: twilio["from_num"].as_ref().unwrap().to_owned(),
  twilio_number_sid: twilio["from_num_sid"].as_ref().unwrap().to_owned(),
  }
}