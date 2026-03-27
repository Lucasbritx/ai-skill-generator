mod app;
mod events;
mod skill;
mod ui;

use std::io::{stdout, Write};

use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = std::io::stdout().flush();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    if app.saved {
        println!("\n✅ Skill saved: {}.md", app.skill.kebab_case_name());
    } else {
        println!("\n👋 Goodbye!");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            ui::render(app, frame);
        })?;

        events::handle_events(app)?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
