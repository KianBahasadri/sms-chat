use tokio::spawn;
use log::info;
use simple_logging;
use homedir;

// local modules
mod load_creds;
mod listener;
mod ui;
mod app;
use app::App;
use ui::ui;

use crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::sync::mpsc;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::backend::Backend;


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
  let data_file = conf_dir.join("sms-chat.json");
  simple_logging::log_to_file(log_path, log::LevelFilter::Info).expect("Error setting up logging");
  info!("Application Starting");

  let ngrok_authtoken = load_creds::get_ngrok_authtoken(&conf_file);
  info!("Recieved ngrok authentication token");

  let twilio_creds: TwilioConfig = load_creds::load_twilio_configs(&conf_file);
  info!("Recieved Twilio Creds");

  let (sender, reciever): (Sender<(String, String)>, Receiver<(String, String)>) = mpsc::channel();

  let mut app = App::new(twilio_creds.clone());
  app.load_data(data_file.to_path_buf());
  
  spawn(listener::setup_listener(twilio_creds.clone(), ngrok_authtoken.clone(), sender));
  info!("Spawned listener");

  info!("Setting up tui");
  // setup terminal
  enable_raw_mode().unwrap();
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend).unwrap();

  run_app(&mut terminal, &mut app, reciever).await;

  app.save_data(data_file);

  // restore terminal
  disable_raw_mode().unwrap();
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  ).unwrap();
  terminal.show_cursor().unwrap();

}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App, reciever: Receiver<(String, String)>) {
  let (key_sender, key_reciever) = mpsc::channel();
  thread::spawn(move || {
    loop {
      key_sender.send(event::read().unwrap()).unwrap();
    }
  });
  loop {
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    
    match reciever.try_recv() {
      Ok((text, number)) => app.peepee_poopoo(text, number),
      Err(_) => ()
    }
    
    if let Ok(Event::Key(key)) = key_reciever.try_recv() {
      if key.kind == event::KeyEventKind::Release {
        continue;
      }

      if let Some(_) = app.curr_contact {
        match key.code {
          KeyCode::Esc => app.curr_contact = None,
          KeyCode::Char(chr) => app.message.push(chr),
          KeyCode::Backspace => {let _ = app.message.pop();}
          KeyCode::Enter => app.send_message().await,
          _ => (),
        }
      } else {
        match key.code {
          KeyCode::Esc | KeyCode::Char('q') => {
            return;
          }
          KeyCode::Enter => {
            match app.message.is_empty() {
              true => app.open_selected_contact(),
              false => {
                app.new_contact(app.message.clone(), None);
                app.message = String::new();
              }
            };
          }
          KeyCode::Up => app.contact_list.state.select_previous(),
          KeyCode::Down => app.contact_list.state.select_next(),
          KeyCode::Char(chr) => app.message.push(chr),
          KeyCode::Backspace => {let _ = app.message.pop();}
          _ => (),
        }
      }
    }
  }
}
