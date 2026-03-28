use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{AITask, App, FormSection};

pub fn handle_events(app: &mut App) -> color_eyre::Result<()> {
    if app.is_loading {
        return Ok(());
    }

    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            handle_key_event(app, key)?;
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> color_eyre::Result<()> {
    if app.is_loading {
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('q') if app.current_section == FormSection::Preview => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.save_skill() {
                Ok(filename) => {
                    app.status_message = Some(format!("Saved to {}", filename));
                }
                Err(e) => {
                    app.status_message = Some(format!("Error: {}", e));
                }
            }
            return Ok(());
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.pending_ai_task = Some(AITask::Enhance);
            return Ok(());
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.pending_ai_task = Some(AITask::FillEmpty);
            return Ok(());
        }
        _ => {}
    }

    if app.current_section == FormSection::Preview {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
            KeyCode::Left | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                app.prev_section();
            }
            KeyCode::Left | KeyCode::BackTab => app.prev_section(),
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.next_section();
            return Ok(());
        }
        KeyCode::BackTab => {
            app.prev_section();
            return Ok(());
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.next_section();
            return Ok(());
        }
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.prev_section();
            return Ok(());
        }
        _ => {}
    }

    if let Some(textarea) = app.current_textarea() {
        textarea.input(key);
    }

    Ok(())
}
