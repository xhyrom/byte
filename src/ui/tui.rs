use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use rfd::FileDialog;
use std::io::stdout;
use std::path::PathBuf;

pub struct TuiInput {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub quality: f32,
}

pub fn run_tui() -> anyhow::Result<TuiInput> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input_stage = 0;
    let mut source = String::new();
    let mut destination = String::new();
    let mut quality = String::from("80");

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3); 4])
                .split(f.area());

            let labels = [
                "Source Folder",
                "Destination Folder",
                "Quality (0–100)",
                "Press Enter to Confirm",
            ];
            let values = [&source, &destination, &quality, ""];

            for i in 0..3 {
                let title = format!(
                    "{}{}",
                    if i == input_stage { "> " } else { "  " },
                    labels[i]
                );
                let block = Block::default().borders(Borders::ALL).title(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
                let paragraph = Paragraph::new(values[i]).block(block);
                f.render_widget(paragraph, chunks[i]);
            }

            let help = Paragraph::new(Line::from(vec![Span::raw(
                "Tab = Next | Backspace = Del | F1 = Browse | Enter = Confirm",
            )]));
            f.render_widget(help, chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => match input_stage {
                        0 => source.push(c),
                        1 => destination.push(c),
                        2 => quality.push(c),
                        _ => {}
                    },
                    KeyCode::Backspace => match input_stage {
                        0 => {
                            source.pop();
                        }
                        1 => {
                            destination.pop();
                        }
                        2 => {
                            quality.pop();
                        }
                        _ => {}
                    },
                    KeyCode::Tab => {
                        input_stage = (input_stage + 1) % 3;
                    }
                    KeyCode::F(1) => {
                        let folder = FileDialog::new().pick_folder();
                        if let Some(path) = folder {
                            match input_stage {
                                0 => source = path.display().to_string(),
                                1 => destination = path.display().to_string(),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if input_stage < 2 {
                            input_stage += 1;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    let quality_parsed = quality.trim().parse::<f32>().unwrap_or(80.0);

    Ok(TuiInput {
        source: PathBuf::from(source.trim()),
        destination: PathBuf::from(destination.trim()),
        quality: quality_parsed,
    })
}
