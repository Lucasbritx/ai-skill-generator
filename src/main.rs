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
    let prompt = format!(
        "You are an AI skill enhancement expert. Your task is to enhance and improve the following skill definition.\n\n\
         IMPORTANT: You MUST respond with the complete enhanced skill in EXACTLY this Markdown format, with ALL sections included. Do NOT ask questions, do NOT add explanations, do NOT add any text before or after the skill.\n\n\
         Enhancements to make:\n\
         1. Improve clarity and precision of all descriptions\n\
         2. Add missing details to make steps more actionable\n\
         3. Ensure all fields are filled and meaningful\n\
         4. Follow LLM best practices for skill definitions\n\n\
         Current Skill:\n\
         {}\n\n\
         Enhanced Skill (respond ONLY with the skill in Markdown format, starting with ## Skill:):",
        skill.to_markdown()
    );

    let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
        .arg("run")
        .arg("--thinking")
        .arg(&prompt)
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let combined = format!("{}\n{}", stderr, stdout);
    let enhanced = extract_skill_from_output(&combined);

    if enhanced.trim().is_empty() {
        return Err("No response from AI".to_string());
    }

    Ok(enhanced)
}

fn fill_empty_fields_async(skill: &crate::skill::Skill) -> Result<String, String> {
    let prompt = format!(
        "You are an AI skill completion engine. The user has provided a partial AI skill definition. Your task is to intelligently complete ALL empty/missing fields based on the provided information.\n\n\
         CRITICAL INSTRUCTIONS:\n\
         1. You MUST fill in EVERY empty field intelligently based on the name and existing content\n\
         2. Respond ONLY with the complete skill in Markdown format (starting with ## Skill:)\n\
         3. Do NOT ask questions, do NOT add explanations, do NOT add any text before or after the skill\n\
         4. For empty sections, create meaningful content that matches the skill's purpose\n\
         5. Make sure to include ALL these sections: Description, Context, Inputs, Steps, Output, Constraints, Tags\n\n\
         FIELD COMPLETION RULES:\n\
         - Name: Keep as is\n\
         - Description: If empty, write a clear description of what this skill does based on the name\n\
         - Context: List relevant technologies, frameworks, or platforms this skill applies to\n\
         - Inputs: List 2-4 key input parameters with descriptions\n\
         - Steps: Provide 3-5 concrete, actionable steps to accomplish the skill\n\
         - Output: Describe what the skill produces or returns\n\
         - Constraints: List 2-3 important limitations, requirements, or considerations\n\
         - Tags: Create 3-5 relevant tags for categorization\n\n\
         Current Skill (some fields may be empty):\n\
         {}\n\n\
         Complete Skill (respond ONLY with the full skill in Markdown format):",
        skill.to_markdown()
    );

    let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
        .arg("run")
        .arg("--thinking")
        .arg(&prompt)
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let combined = format!("{}\n{}", stderr, stdout);
    let enhanced = extract_skill_from_output(&combined);

    if enhanced.trim().is_empty() {
        return Err("No response from AI".to_string());
    }

    Ok(enhanced)
}

fn extract_skill_from_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut in_skill = false;
    let mut skill_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## Skill:") {
            if in_skill && !skill_lines.is_empty() {
                break;
            }
            in_skill = true;
        }
        if in_skill {
            skill_lines.push(line);
        }
    }

    skill_lines.join("\n")
}
