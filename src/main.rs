mod app;
mod events;
mod skill;
mod ui;

use std::io::{stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{AITask, App};

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

        // Handle pending AI task spawning
        if let Some(task) = app.pending_ai_task.take() {
            spawn_ai_task(app, task);
        }

        // Poll for AI task results
        if let Some(rx) = app.task_result_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    app.is_loading = false;
                    app.loading_message = None;

                    match result {
                        Ok(enhanced) => {
                            app.parse_enhanced_skill(&enhanced);
                            app.status_message = Some("✨ Done!".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("AI Error: {}", e));
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Task still running, put receiver back
                    app.task_result_rx = Some(rx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected unexpectedly
                    app.is_loading = false;
                    app.loading_message = None;
                    app.status_message = Some("AI Error: Channel disconnected".to_string());
                }
            }
        }

        events::handle_events(app)?;

        if app.should_quit {
            break;
        }

        // Small sleep to avoid busy waiting
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn spawn_ai_task(app: &mut App, task: AITask) {
    app.is_loading = true;

    let loading_msg = match task {
        AITask::Enhance => "🤖 Enhancing with AI...".to_string(),
        AITask::FillEmpty => "🤖 Filling empty fields...".to_string(),
    };
    app.loading_message = Some(loading_msg.clone());
    app.status_message = Some(loading_msg);

    let skill_data = app.skill.clone();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = match task {
            AITask::Enhance => enhance_skill_async(&skill_data),
            AITask::FillEmpty => fill_empty_fields_async(&skill_data),
        };
        let _ = tx.send(result);
    });

    app.task_result_rx = Some(rx);
}

fn enhance_skill_async(skill: &crate::skill::Skill) -> Result<String, String> {
    let has_empty_fields = skill.description.is_empty()
        || skill.context.is_empty()
        || skill.inputs.is_empty()
        || skill.steps.is_empty()
        || skill.output.is_empty()
        || skill.constraints.is_empty()
        || skill.tags.is_empty();

    let skill_markdown = if has_empty_fields {
        skill.to_markdown_with_empty_sections()
    } else {
        skill.to_markdown()
    };

    let original_name = skill.name.clone();

    // Simple prompt - just ask to enhance the skill
    let prompt = if has_empty_fields {
        format!(
            "Enhance this skill definition. For any [To be filled] fields, make reasonable assumptions. Keep the skill name as '{}':\n\n{}",
            original_name, skill_markdown
        )
    } else {
        format!(
            "Enhance and improve this skill definition. Keep the skill name as '{}':\n\n{}",
            original_name, skill_markdown
        )
    };

    let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
        .arg("run")
        .arg(&prompt)
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let combined = format!("{}\n{}", stderr, stdout);
    let mut enhanced = extract_skill_from_output(&combined);

    if enhanced.trim().is_empty() {
        return Err("No response from AI".to_string());
    }

    // FALLBACK: If AI changed the skill name, restore the original
    if !enhanced.contains(&format!("## Skill: {}", original_name)) {
        if let Some(pos) = enhanced.find("## Skill:") {
            let header_end = enhanced[pos..].find('\n').unwrap_or(enhanced.len() - pos);
            let new_header = format!("## Skill: {}", original_name);
            enhanced = format!("{}\n{}", new_header, &enhanced[pos + header_end..]);
        }
    }

    Ok(enhanced)
}

fn fill_empty_fields_async(skill: &crate::skill::Skill) -> Result<String, String> {
    let skill_markdown = skill.to_markdown_with_empty_sections();
    let original_name = skill.name.clone();

    // Simple prompt - just ask to fill the blanks
    // If fields are empty, ask AI to make reasonable assumptions
    let prompt = format!(
        "Complete this skill definition. For any [To be filled] fields, make reasonable assumptions. Keep the skill name as '{}':\n\n{}",
        original_name, skill_markdown
    );

    let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
        .arg("run")
        .arg(&prompt)
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let combined = format!("{}\n{}", stderr, stdout);
    let mut enhanced = extract_skill_from_output(&combined);

    if enhanced.trim().is_empty() {
        return Err("No response from AI".to_string());
    }

    // FALLBACK: If AI changed the skill name, restore the original
    if !enhanced.contains(&format!("## Skill: {}", original_name)) {
        if let Some(pos) = enhanced.find("## Skill:") {
            let header_end = enhanced[pos..].find('\n').unwrap_or(enhanced.len() - pos);
            let new_header = format!("## Skill: {}", original_name);
            enhanced = format!("{}\n{}", new_header, &enhanced[pos + header_end..]);
        }
    }

    Ok(enhanced)
}

fn extract_skill_from_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut in_skill = false;
    let mut in_code_block = false;
    let mut skill_lines = Vec::new();
    let mut found_skill_header = false;

    for line in lines.iter() {
        let trimmed = line.trim();

        // Handle markdown code blocks (```markdown or ``` or ```text)
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block - stop collecting if we found a skill
                if found_skill_header && !skill_lines.is_empty() {
                    break;
                }
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
            }
            continue;
        }

        // Look for skill header
        if trimmed.starts_with("## Skill:") {
            // If we already found a skill, stop here (don't include duplicates)
            if found_skill_header && !skill_lines.is_empty() {
                break;
            }
            in_skill = true;
            found_skill_header = true;
        }

        // Collect lines once we found the skill header (skip empty lines at start)
        if in_skill && !trimmed.is_empty() {
            skill_lines.push(*line);
        }
    }

    skill_lines.join("\n")
}
