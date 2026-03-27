mod app;
mod events;
mod skill;
mod ui;

use std::io::stdout;

use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Handle any errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        return Err(e);
    }

    // Print farewell message
    if app.saved {
        println!("\n✅ Skill saved successfully!");
        println!("   File: {}.md", app.skill.kebab_case_name());
    } else {
        println!("\n👋 Thanks for using AI Skill Generator!");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            ui::render(app, frame);
        })?;

        // Handle events
        events::handle_events(app)?;

        // Check for quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
