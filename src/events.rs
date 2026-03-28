use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{App, FormSection};

/// Handle all input events
pub fn handle_events(app: &mut App) -> color_eyre::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            handle_key_event(app, key)?;
        }
    }
    Ok(())
}

/// Handle keyboard events
fn handle_key_event(app: &mut App, key: KeyEvent) -> color_eyre::Result<()> {
    // Global keybindings
    match key.code {
        // Quit
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
        // Save
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
        // AI Enhancement (improve existing)
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.enhance_with_ai() {
                Ok(enhanced) => {
                    app.parse_enhanced_skill(&enhanced);
                    app.status_message = Some("✨ Skill enhanced with AI!".to_string());
                }
                Err(e) => {
                    app.status_message = Some(format!("AI Error: {}", e));
                }
            }
            return Ok(());
        }
        // AI Fill (fill empty fields)
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.fill_empty_fields() {
                Ok(enhanced) => {
                    app.parse_enhanced_skill(&enhanced);
                    app.status_message = Some("✨ Empty fields filled!".to_string());
                }
                Err(e) => {
                    app.status_message = Some(format!("AI Error: {}", e));
                }
            }
            return Ok(());
        }
        _ => {}
    }

    // Preview section specific handling
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

    // Navigation between sections
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

    // Pass other keys to the textarea
    if let Some(textarea) = app.current_textarea() {
        textarea.input(key);
    }

    Ok(())
}
