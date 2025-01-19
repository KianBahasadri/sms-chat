use log::info;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph}, Frame};
use crate::App;
use crate::TwilioConfig;

pub async fn send_msg(twilio: &TwilioConfig, msg: &str, to_number: &str) {
  info!("send_msg called with: {}", msg);
  let client = reqwest::Client::new();

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

pub fn ui(frame: &mut Frame, app: &mut App) {
  let chunks = Layout::default()
  .direction(Direction::Vertical)
  .constraints([
      Constraint::Length(3),
      Constraint::Min(1),
      Constraint::Length(3),
  ])
  .split(frame.area());
  
  render_header(frame, app, chunks[0]);

  if app.curr_contact == None {
    render_body_contacts(frame, app, chunks[1]);
  } else {
    render_body_messages(frame, app, chunks[1]);
  }

  render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, chunk:Rect ) {
  let title_block = Block::default()
  .borders(Borders::ALL)
  .style(Style::default());
  
  let title = Paragraph::new(Text::styled(
    format!("sms-chat: {} contacts", app.contact_list.items.len()),
    Style::default().fg(Color::White).bold(),
  ))
  .centered()
  .block(title_block)
  .bg(Color::Black);

  frame.render_widget(title, chunk);
}

fn render_body_contacts(frame: &mut Frame, app: &mut App, chunk:Rect ) {
  let mut list_items = Vec::<ListItem>::new();

    for contact in &app.contact_list.items {
      let line = Line::from(vec![
        format!(" {:<16}", contact.number).into(),
        format!("{:<16}", contact.name.as_ref().unwrap_or(&String::new())).into(),
        format!("{} messages", contact.messages.len()).into(),
      ]);
      list_items.push(ListItem::new(line));
    }
  
  let list = List::new(list_items)
    .bg(Color::Black)
    .highlight_style(Style::new().fg(Color::Cyan).bold())
    .highlight_spacing(HighlightSpacing::Always)
    .highlight_symbol(">");

  frame.render_stateful_widget(list, chunk, &mut app.contact_list.state);
}

fn render_body_messages(frame: &mut Frame, app: &App, chunk:Rect ) {
  let mut list_items = Vec::<ListItem>::new();
  let contact = &app.contact_list.items[app.curr_contact.unwrap()];
  let name_len= contact.to_string().len();
  
  for message in &app.contact_list.items[app.curr_contact.unwrap()].messages {
    let sender = match message.from_self {
      true => String::from("You"),
      false => contact.to_string(),
    };
    let style = match message.from_self {
        true => Style::new().dark_gray().bold(),
        false => Style::new().gray(),
    };
    let line = Line::from(vec![
      format!("{:<16}", message.date).yellow(),
      Span::styled(format!("{:>name_len$}: ", sender), style),
      Span::styled(&message.text, style),
    ]);
    list_items.push(ListItem::new(line));
    }
  let list = List::new(list_items).bg(Color::Black);
  
frame.render_widget(list, chunk);
}

fn render_footer(frame: &mut Frame, app: &App, chunk:Rect ) {
  let style = Style::default().fg(Color::White).bold();
  let current_keys_hint = match app.message.is_empty() {
    false => Span::styled(&app.message, style),
    true => match app.curr_contact {
      None => Span::styled("(Esc) / (q), (Up) & (Down), (Enter) -- Type to add a new number", style),
      Some(_) => Span::styled("(Esc), Type & (Enter)", style),
    }
  };
        
  let key_notes_footer = Paragraph::new(
  Line::from(current_keys_hint))
  .block(Block::default()
  .borders(Borders::ALL))
  .centered()
  .bg(Color::Black);

  frame.render_widget(key_notes_footer, chunk);
}