use std::io;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Borders, List, ListItem};
use tui::{Frame, Terminal};

use pueue::platform::socket::Socket;
use pueue::settings::Settings;
use pueue::state::State;

use crate::cli::CliArguments;
use crate::commands::get_state;

pub struct App {
    pub selected_task: usize,
}

pub async fn run(settings: Settings, opt: CliArguments, mut socket: Socket) -> Result<()> {
    // Tui initialization
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    enable_raw_mode()?;

    // Create app to store some state
    let mut app = App { selected_task: 0 };
    let update_time = Duration::from_millis(2000);
    let mut first = true;

    loop {
        // `poll()` waits for an `Event` for a given time period
        if !first && poll(update_time)? {
            match read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('j') | KeyCode::Down => app.selected_task -= 1,
                    KeyCode::Char('k') | KeyCode::Up => app.selected_task += 1,
                    KeyCode::Char('q') => break,
                    _ => (),
                },
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        }
        let state = get_state(&mut socket).await?;

        terminal.draw(|frame| {
            // Split the layout into the task list on the left and the rest
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(10), Constraint::Min(20)].as_ref())
                .split(frame.size());

            // Draw the task list
            draw_task_list(&mut app, &state, &chunks[0], frame);
        })?;
        first = false;
    }

    disable_raw_mode()?;

    Ok(())
}

pub fn draw_task_list(
    app: &mut App,
    state: &State,
    area: &Rect,
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
) {
    let mut tasks: Vec<ListItem> = Vec::new();
    for id in state.tasks.keys() {
        // Define highlighted style for the selected task.
        let text = format!("Task {}", id);
        let mut list_item = ListItem::new(text);
        if *id == app.selected_task {
            list_item = list_item.style(Style::default().add_modifier(Modifier::BOLD));
        }
        tasks.push(list_item);
    }

    frame.render_widget(List::new(tasks), *area);
}
