use crate::skill::{Input, Skill};
use tui_textarea::TextArea;

/// Enum representing the different form sections/steps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormSection {
    Name,
    Description,
    Context,
    Inputs,
    Steps,
    Output,
    Constraints,
    Tags,
    Preview,
}

impl FormSection {
    pub fn title(&self) -> &'static str {
        match self {
            FormSection::Name => "Skill Name",
            FormSection::Description => "Description",
            FormSection::Context => "Context (Technologies/Frameworks)",
            FormSection::Inputs => "Inputs",
            FormSection::Steps => "Steps",
            FormSection::Output => "Expected Output",
            FormSection::Constraints => "Constraints",
            FormSection::Tags => "Tags",
            FormSection::Preview => "Preview & Save",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            FormSection::Name => "Enter a descriptive name for your skill (will be converted to kebab-case)",
            FormSection::Description => "Provide a clear and concise explanation of what the skill does",
            FormSection::Context => "List technologies, frameworks, or environment assumptions (one per line)",
            FormSection::Inputs => "Define input parameters (format: name: description, one per line)",
            FormSection::Steps => "List actionable steps to accomplish the skill (one per line)",
            FormSection::Output => "Describe what the skill produces",
            FormSection::Constraints => "List any limitations or requirements (one per line)",
            FormSection::Tags => "Add tags for categorization (space or comma separated, # optional)",
            FormSection::Preview => "Review your skill and save it",
        }
    }

    pub fn next(&self) -> Option<FormSection> {
        match self {
            FormSection::Name => Some(FormSection::Description),
            FormSection::Description => Some(FormSection::Context),
            FormSection::Context => Some(FormSection::Inputs),
            FormSection::Inputs => Some(FormSection::Steps),
            FormSection::Steps => Some(FormSection::Output),
            FormSection::Output => Some(FormSection::Constraints),
            FormSection::Constraints => Some(FormSection::Tags),
            FormSection::Tags => Some(FormSection::Preview),
            FormSection::Preview => None,
        }
    }

    pub fn prev(&self) -> Option<FormSection> {
        match self {
            FormSection::Name => None,
            FormSection::Description => Some(FormSection::Name),
            FormSection::Context => Some(FormSection::Description),
            FormSection::Inputs => Some(FormSection::Context),
            FormSection::Steps => Some(FormSection::Inputs),
            FormSection::Output => Some(FormSection::Steps),
            FormSection::Constraints => Some(FormSection::Output),
            FormSection::Tags => Some(FormSection::Constraints),
            FormSection::Preview => Some(FormSection::Tags),
        }
    }

    pub fn all() -> Vec<FormSection> {
        vec![
            FormSection::Name,
            FormSection::Description,
            FormSection::Context,
            FormSection::Inputs,
            FormSection::Steps,
            FormSection::Output,
            FormSection::Constraints,
            FormSection::Tags,
            FormSection::Preview,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            FormSection::Name => 0,
            FormSection::Description => 1,
            FormSection::Context => 2,
            FormSection::Inputs => 3,
            FormSection::Steps => 4,
            FormSection::Output => 5,
            FormSection::Constraints => 6,
            FormSection::Tags => 7,
            FormSection::Preview => 8,
        }
    }
}

/// Main application state
pub struct App<'a> {
    /// Current form section
    pub current_section: FormSection,
    /// Text areas for each section
    pub text_areas: Vec<TextArea<'a>>,
    /// The skill being built
    pub skill: Skill,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Status message
    pub status_message: Option<String>,
    /// Whether we're in editing mode
    pub editing: bool,
    /// Scroll offset for preview
    pub preview_scroll: u16,
    /// Whether save was successful
    pub saved: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut text_areas: Vec<TextArea<'a>> = Vec::new();

        // Create a text area for each section (except Preview)
        for section in FormSection::all() {
            if section != FormSection::Preview {
                let mut ta = TextArea::default();
                ta.set_cursor_line_style(ratatui::style::Style::default());
                text_areas.push(ta);
            }
        }

        Self {
            current_section: FormSection::Name,
            text_areas,
            skill: Skill::default(),
            should_quit: false,
            status_message: None,
            editing: true,
            preview_scroll: 0,
            saved: false,
        }
    }

    /// Get the current text area
    pub fn current_textarea(&mut self) -> Option<&mut TextArea<'a>> {
        let idx = self.current_section.index();
        if idx < self.text_areas.len() {
            Some(&mut self.text_areas[idx])
        } else {
            None
        }
    }

    /// Move to next section
    pub fn next_section(&mut self) {
        self.sync_skill_from_textarea();
        if let Some(next) = self.current_section.next() {
            self.current_section = next;
            self.status_message = None;
        }
    }

    /// Move to previous section
    pub fn prev_section(&mut self) {
        self.sync_skill_from_textarea();
        if let Some(prev) = self.current_section.prev() {
            self.current_section = prev;
            self.status_message = None;
        }
    }

    /// Sync skill data from current textarea
    pub fn sync_skill_from_textarea(&mut self) {
        let idx = self.current_section.index();
        if idx >= self.text_areas.len() {
            return;
        }

        let content: String = self.text_areas[idx].lines().join("\n");

        match self.current_section {
            FormSection::Name => {
                self.skill.name = content.trim().to_string();
            }
            FormSection::Description => {
                self.skill.description = content.trim().to_string();
            }
            FormSection::Context => {
                self.skill.context = content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            FormSection::Inputs => {
                self.skill.inputs = content
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() {
                            return None;
                        }
                        if let Some((name, desc)) = line.split_once(':') {
                            Some(Input {
                                name: name.trim().to_string(),
                                description: desc.trim().to_string(),
                            })
                        } else {
                            Some(Input {
                                name: line.to_string(),
                                description: String::new(),
                            })
                        }
                    })
                    .collect();
            }
            FormSection::Steps => {
                self.skill.steps = content
                    .lines()
                    .map(|s| {
                        let s = s.trim();
                        // Remove leading numbers like "1. " or "1) "
                        if let Some(rest) = s.strip_prefix(|c: char| c.is_ascii_digit()) {
                            rest.trim_start_matches(['.', ')', ' ']).trim().to_string()
                        } else {
                            s.to_string()
                        }
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            FormSection::Output => {
                self.skill.output = content.trim().to_string();
            }
            FormSection::Constraints => {
                self.skill.constraints = content
                    .lines()
                    .map(|s| s.trim().trim_start_matches('-').trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            FormSection::Tags => {
                self.skill.tags = content
                    .replace(',', " ")
                    .split_whitespace()
                    .map(|s| s.trim_start_matches('#').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            FormSection::Preview => {}
        }
    }

    /// Scroll preview up
    pub fn scroll_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    /// Scroll preview down
    pub fn scroll_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    /// Save the skill to a file
    pub fn save_skill(&mut self) -> color_eyre::Result<String> {
        self.sync_skill_from_textarea();

        if !self.skill.is_valid() {
            return Err(color_eyre::eyre::eyre!(
                "Skill must have at least a name and description"
            ));
        }

        let filename = format!("{}.md", self.skill.kebab_case_name());
        let content = self.skill.to_markdown();

        std::fs::write(&filename, &content)?;
        self.saved = true;
        self.status_message = Some(format!("Saved to {}", filename));

        Ok(filename)
    }

    /// Get progress percentage
    pub fn progress(&self) -> f64 {
        let total = FormSection::all().len() as f64;
        let current = (self.current_section.index() + 1) as f64;
        current / total
    }
}

impl Default for App<'_> {
    fn default() -> Self {
        Self::new()
    }
}
