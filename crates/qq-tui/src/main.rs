use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use qq_core::{ClientCommand, ClientCore, ConnectionConfig, ConversationId, ConversationView, MessageView};
use qq_napcat::NapCatGateway;
use qq_render::{PlainTextRenderer, RichTextRenderer};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

#[derive(Default)]
struct App {
    conversations: Vec<ConversationView>,
    messages: Vec<MessageView>,
    selected: usize,
    input: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let core = ClientCore::new();
    let gateway = NapCatGateway::new();
    let mut gateway_events = gateway.subscribe();
    let bridge_core = core.clone();
    tokio::spawn(async move {
        while let Ok(event) = gateway_events.recv().await {
            let _ = bridge_core.dispatch_event(event).await;
        }
    });

    let endpoint = std::env::var("NAPCAT_WS").unwrap_or_else(|_| ConnectionConfig::default().endpoint);
    gateway.execute(ClientCommand::Connect(ConnectionConfig {
        endpoint,
        access_token: std::env::var("NAPCAT_TOKEN").ok(),
        reconnect: true,
    })).await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_ui(&mut terminal, core, gateway).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

async fn run_ui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, core: ClientCore, gateway: NapCatGateway) -> Result<()> {
    let mut app = App {
        status: "connecting".to_owned(),
        ..App::default()
    };

    loop {
        refresh(&mut app, &core).await;
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(80))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => break,
                KeyCode::Up => {
                    app.selected = app.selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    if !app.conversations.is_empty() {
                        app.selected = (app.selected + 1).min(app.conversations.len() - 1);
                    }
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    if let Some(conversation_id) = selected_conversation(&app) {
                        let text = app.input.trim().to_owned();
                        if !text.is_empty() {
                            gateway.execute(ClientCommand::SendTextMessage { conversation_id, text }).await?;
                            app.input.clear();
                        }
                    }
                }
                KeyCode::Char(value) => app.input.push(value),
                _ => {}
            }
        }
    }

    Ok(())
}

async fn refresh(app: &mut App, core: &ClientCore) {
    app.conversations = core.conversations().await;
    if let Some(conversation_id) = selected_conversation(app) {
        app.messages = core.timeline(&conversation_id).await.messages;
    }
}

fn selected_conversation(app: &App) -> Option<ConversationId> {
    app.conversations.get(app.selected).map(|conversation| conversation.id.clone())
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3), Constraint::Length(1)])
        .split(frame.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(20)])
        .split(root[0]);

    let conversations = app.conversations.iter().enumerate().map(|(index, conversation)| {
        let marker = if index == app.selected { ">" } else { " " };
        ListItem::new(Line::from(vec![
            Span::styled(marker, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(&conversation.title, Style::default().add_modifier(Modifier::BOLD)),
        ]))
    });
    frame.render_widget(
        List::new(conversations).block(Block::default().title("Conversations").borders(Borders::ALL)),
        body[0],
    );

    let lines = app.messages.iter().flat_map(|message| {
        let text = PlainTextRenderer::render(&message.nodes);
        [
            Line::from(Span::styled(format!("{}:", message.sender_id), Style::default().fg(Color::Yellow))),
            Line::from(text),
            Line::from(""),
        ]
    }).collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().title("Messages").borders(Borders::ALL)),
        body[1],
    );

    frame.render_widget(
        Paragraph::new(app.input.as_str()).block(Block::default().title("Input").borders(Borders::ALL)),
        root[1],
    );
    frame.render_widget(
        Paragraph::new(format!("{} | Esc quit | Enter send | NAPCAT_WS configures endpoint", app.status)),
        root[2],
    );
}
