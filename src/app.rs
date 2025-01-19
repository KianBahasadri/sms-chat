// Everything that has to do with app state gets dealt with in this file

use core::fmt;
use std::{fs::{self}, path::PathBuf};
use chrono::Local;
use log::info;
use ratatui::widgets::ListState;
use serde::{Serialize, Deserialize};
use crate::{ui::send_msg, TwilioConfig};

#[derive(Serialize, Deserialize)]
pub struct Message {
  pub from_self: bool,
  pub text: String,
  pub date: String,
}

#[derive(Serialize, Deserialize)]
pub struct Contact {
  pub number: String,
  pub name: Option<String>,
  pub messages: Vec<Message>,
}

impl fmt::Display for Contact {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name.as_deref().unwrap_or(&self.number))
  }
}

pub struct ContactList {
  pub items: Vec<Contact>,
  pub state: ListState,
}

pub struct App {
  pub contact_list: ContactList,
  pub curr_contact: Option<usize>,
  pub message: String,
  pub twilio_config: TwilioConfig,
}

impl App {
  pub fn new(twilio_config: TwilioConfig) -> Self {
    Self {
      contact_list: ContactList { items: Vec::new(), state: ListState::default() },
      curr_contact: None,
      message: String::new(),
      twilio_config,
      }
  }

  pub fn new_contact(&mut self, number: String, name: Option<String>) {
    self.contact_list.items.push(Contact {
      number,
      name,
      messages: Vec::new(),
    })
  }

  pub fn peepee_poopoo(&mut self, text: String, number: String) {
    match self.contact_list.items.iter().position(|contact| contact.number == number) {
      Some(index) => {
        info!("{} already exists with index {}", &number, index);
        self.recieve_message(text, index);
      } None => {
        info!("{} does not already exist, creating new contact", &number);
        self.new_contact(number, None);
        let num_contacts = self.contact_list.items.len();
        self.recieve_message(text, num_contacts-1);
        info!("registered new number and text");
      }
    }
  }

  pub fn recieve_message(&mut self, text: String, contact_index: usize) {
    let contact = self.contact_list.items.get_mut(contact_index).unwrap();
    contact.messages.push(Message {
      from_self: false,
      text,
      date: Local::now().format("%b %d %I:%M %p").to_string(),
    });    
  }

  pub async fn send_message(&mut self) {
    self.contact_list.items[self.curr_contact.unwrap()].messages.push(Message {
      from_self: true,
      text: self.message.clone(),
      date: Local::now().format("%b %d %I:%M %p").to_string(),
    });
    send_msg(&self.twilio_config, &self.message, &self.contact_list.items[self.curr_contact.unwrap()].number).await;
    self.message = String::new();
  }

  pub fn open_selected_contact(& mut self) {
    if let Some(index) = self.contact_list.state.selected() {
      self.curr_contact = Some(index);
    }
  }

  pub fn save_data(&mut self, data_file: PathBuf) {
    let serialized = serde_json::to_string(&self.contact_list.items).unwrap();
    fs::write(&data_file, serialized).unwrap();
    info!("Data saved successfully to {}", data_file.display());
  }

  pub fn load_data(&mut self, data_file: PathBuf) {
    if !data_file.exists() {
      info!("The file {} does not exist.", data_file.display());
    } else {
      let serialized = fs::read(&data_file).unwrap();
      let serialized_str = String::from_utf8_lossy(&serialized);
      self.contact_list.items = serde_json::from_str(&serialized_str).unwrap();
      info!("Data loaded from {}", data_file.display());
    }
  }
}
