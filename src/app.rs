use crate::skill::{Input, Skill};
use std::sync::mpsc::Receiver;
use tui_textarea::TextArea;

/// Enum representing AI tasks to be run
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AITask {
    Enhance,
    FillEmpty,
}

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
            FormSection::Name => {
                "Enter a descriptive name for your skill (will be converted to kebab-case)"
            }
            FormSection::Description => {
                "Provide a clear and concise explanation of what the skill does"
            }
            FormSection::Context => {
                "List technologies, frameworks, or environment assumptions (one per line)"
            }
            FormSection::Inputs => {
                "Define input parameters (format: name: description, one per line)"
            }
            FormSection::Steps => "List actionable steps to accomplish the skill (one per line)",
            FormSection::Output => "Describe what the skill produces",
            FormSection::Constraints => "List any limitations or requirements (one per line)",
            FormSection::Tags => {
                "Add tags for categorization (space or comma separated, # optional)"
            }
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
    /// Whether AI is loading/processing
    pub is_loading: bool,
    /// Loading message
    pub loading_message: Option<String>,
    /// Pending AI task to run
    pub pending_ai_task: Option<AITask>,
    /// Receiver for AI task results
    pub task_result_rx: Option<Receiver<Result<String, String>>>,
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
            is_loading: false,
            loading_message: None,
            pending_ai_task: None,
            task_result_rx: None,
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

    /// Enhance skill using OpenCode AI CLI
    pub fn enhance_with_ai(&mut self) -> color_eyre::Result<String> {
        self.sync_skill_from_textarea();

        if self.skill.name.is_empty() && self.skill.description.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Skill must have at least a name or description"
            ));
        }

        let has_empty_fields = self.skill.context.is_empty()
            || self.skill.inputs.is_empty()
            || self.skill.steps.is_empty()
            || self.skill.output.is_empty()
            || self.skill.constraints.is_empty()
            || self.skill.tags.is_empty();

        let prompt = if has_empty_fields {
            format!(
                r#"You are an AI skill enhancement engine. Enhance and complete the following skill by:
1. Improving clarity and descriptions where they exist
2. Adding missing details for empty fields (context, inputs, steps, output, constraints, tags)
3. Making steps more actionable
4. Ensuring best practices for LLM-based systems

Current skill (some fields may be empty - FILL THEM IN based on the name/description):
{}

Output ONLY the enhanced skill in exact same Markdown format. Do NOT ask questions. Do NOT include explanations. Just output the complete enhanced skill with ALL fields populated."#,
                self.skill.to_markdown()
            )
        } else {
            format!(
                r#"You are an AI skill enhancement engine. Enhance the following skill by improving clarity, adding missing details, making steps more actionable, and ensuring best practices for LLM-based systems.

Output ONLY the enhanced skill in exact same Markdown format. Do NOT ask questions. Do NOT include explanations. Just output the enhanced skill.

---

{}

---

Enhanced skill:"#,
                self.skill.to_markdown()
            )
        };

        self.status_message = Some("🤖 Enhancing with AI...".to_string());

        let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
            .arg("run")
            .arg("--thinking")
            .arg(&prompt)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let combined = format!("{}\n{}", stderr, stdout);

        let enhanced = Self::extract_skill_from_output(&combined);

        if enhanced.trim().is_empty() {
            return Err(color_eyre::eyre::eyre!("No response from AI"));
        }

        self.status_message = Some("✨ Skill enhanced!".to_string());

        Ok(enhanced.to_string())
    }

    /// Fill empty fields using OpenCode AI CLI
    pub fn fill_empty_fields(&mut self) -> color_eyre::Result<String> {
        self.sync_skill_from_textarea();

        if self.skill.name.is_empty() && self.skill.description.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Skill must have at least a name or description"
            ));
        }

        let prompt = format!(
            r#"You are an AI skill completion engine. The user has provided a partial skill with some fields empty.

Based ONLY on the provided fields (name, description, context, inputs, steps, output, constraints, tags), fill in the missing/empty fields. 

Do NOT make up technologies or details that aren't implied by the existing fields. Keep it simple and accurate.

Current skill (fields with "N/A" or that are empty need to be filled):
{}

Output ONLY the completed skill in exact same Markdown format. Do NOT ask questions. Do NOT include explanations. Just output the skill with empty fields filled in."#,
            self.skill.to_markdown()
        );

        self.status_message = Some("🤖 Filling empty fields...".to_string());

        let output = std::process::Command::new("/Users/lucasxavier/.opencode/bin/opencode")
            .arg("run")
            .arg("--thinking")
            .arg(&prompt)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let combined = format!("{}\n{}", stderr, stdout);

        let enhanced = Self::extract_skill_from_output(&combined);

        if enhanced.trim().is_empty() {
            return Err(color_eyre::eyre::eyre!("No response from AI"));
        }

        self.status_message = Some("✨ Empty fields filled!".to_string());

        Ok(enhanced.to_string())
    }

    /// Extract skill content from AI output
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

    /// Parse enhanced skill from AI response
    pub fn parse_enhanced_skill(&mut self, enhanced: &str) {
        let lines: Vec<&str> = enhanced.lines().collect();
        let mut current_section: Option<&str> = None;
        let mut name = String::new();
        let mut description = String::new();
        let mut context = Vec::new();
        let mut inputs = Vec::new();
        let mut steps = Vec::new();
        let mut output = String::new();
        let mut constraints = Vec::new();
        let mut tags = Vec::new();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Check for section headers
            if trimmed.starts_with("## Skill:") {
                name = trimmed.trim_start_matches("## Skill:").trim().to_string();
                current_section = None;
                i += 1;
                continue;
            }

            // Match section headers
            match trimmed {
                "### Description" => {
                    current_section = Some("description");
                    i += 1;
                    continue;
                }
                "### Context" => {
                    current_section = Some("context");
                    i += 1;
                    continue;
                }
                "### Inputs" => {
                    current_section = Some("inputs");
                    i += 1;
                    continue;
                }
                "### Steps" => {
                    current_section = Some("steps");
                    i += 1;
                    continue;
                }
                "### Output" => {
                    current_section = Some("output");
                    i += 1;
                    continue;
                }
                "### Constraints" => {
                    current_section = Some("constraints");
                    i += 1;
                    continue;
                }
                "### Tags" => {
                    current_section = Some("tags");
                    i += 1;
                    continue;
                }
                _ if trimmed.starts_with("### ") => {
                    // Any other section header
                    current_section = None;
                    i += 1;
                    continue;
                }
                _ => {}
            }

            // Process content based on current section
            if let Some(section) = current_section {
                match section {
                    "description" => {
                        // Collect all non-empty lines until next section
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            if !description.is_empty() {
                                description.push('\n');
                            }
                            description.push_str(trimmed);
                        }
                    }
                    "context" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                                let item = trimmed
                                    .trim_start_matches('-')
                                    .trim_start_matches('*')
                                    .trim()
                                    .to_string();
                                if !item.is_empty() {
                                    context.push(item);
                                }
                            }
                        }
                    }
                    "inputs" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                                let input_line = trimmed
                                    .trim_start_matches('-')
                                    .trim_start_matches('*')
                                    .trim();

                                // Remove bold markers ** if present
                                let input_line = input_line.trim_matches('*');

                                if let Some((name_part, desc_part)) = input_line.split_once(':') {
                                    inputs.push(Input {
                                        name: name_part.trim().trim_matches('*').to_string(),
                                        description: desc_part.trim().to_string(),
                                    });
                                } else if !input_line.is_empty() {
                                    inputs.push(Input {
                                        name: input_line.to_string(),
                                        description: String::new(),
                                    });
                                }
                            }
                        }
                    }
                    "steps" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            // Handle both "1. step" and "- step" formats
                            let step_text =
                                if trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                    // Numbered format: "1. text" or "1) text"
                                    trimmed
                                        .trim_start_matches(|c: char| c.is_ascii_digit())
                                        .trim_start_matches('.')
                                        .trim_start_matches(')')
                                        .trim()
                                        .to_string()
                                } else if trimmed.starts_with('-') || trimmed.starts_with('*') {
                                    // Bullet format: "- text" or "* text"
                                    trimmed
                                        .trim_start_matches('-')
                                        .trim_start_matches('*')
                                        .trim()
                                        .to_string()
                                } else {
                                    trimmed.to_string()
                                };

                            if !step_text.is_empty() {
                                steps.push(step_text);
                            }
                        }
                    }
                    "output" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            if !output.is_empty() {
                                output.push('\n');
                            }
                            output.push_str(trimmed);
                        }
                    }
                    "constraints" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                                let item = trimmed
                                    .trim_start_matches('-')
                                    .trim_start_matches('*')
                                    .trim()
                                    .to_string();
                                if !item.is_empty() {
                                    constraints.push(item);
                                }
                            }
                        }
                    }
                    "tags" => {
                        if !trimmed.is_empty() && !trimmed.starts_with("### ") {
                            // Split by whitespace or comma
                            for tag_str in trimmed.split(|c: char| c.is_whitespace() || c == ',') {
                                let tag = tag_str.trim().trim_start_matches('#').trim_matches('*');
                                if !tag.is_empty() {
                                    tags.push(tag.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            i += 1;
        }

        self.skill.name = name;
        self.skill.description = description;
        self.skill.context = context;
        self.skill.inputs = inputs;
        self.skill.steps = steps;
        self.skill.output = output;
        self.skill.constraints = constraints;
        self.skill.tags = tags;

        self.populate_textareas_from_skill();
    }

    /// Populate textareas from skill data (after AI enhancement)
    pub fn populate_textareas_from_skill(&mut self) {
        // Create new textareas with skill data
        let ta_name = TextArea::new(vec![self.skill.name.clone()]);
        let ta_desc = TextArea::new(vec![self.skill.description.clone()]);
        let ta_context = TextArea::new(self.skill.context.clone());

        let input_text: Vec<String> = self
            .skill
            .inputs
            .iter()
            .map(|i| {
                if i.description.is_empty() {
                    i.name.clone()
                } else {
                    format!("{}: {}", i.name, i.description)
                }
            })
            .collect();
        let ta_inputs = TextArea::new(input_text);

        let ta_steps = TextArea::new(self.skill.steps.clone());
        let ta_output = TextArea::new(vec![self.skill.output.clone()]);
        let ta_constraints = TextArea::new(self.skill.constraints.clone());
        let ta_tags = TextArea::new(vec![self.skill.tags.join(" ")]);

        self.text_areas = vec![
            ta_name,
            ta_desc,
            ta_context,
            ta_inputs,
            ta_steps,
            ta_output,
            ta_constraints,
            ta_tags,
        ];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_section_navigation() {
        assert_eq!(FormSection::Name.next(), Some(FormSection::Description));
        assert_eq!(FormSection::Description.prev(), Some(FormSection::Name));
        assert_eq!(FormSection::Preview.next(), None);
        assert_eq!(FormSection::Name.prev(), None);
    }

    #[test]
    fn test_form_section_titles() {
        assert_eq!(FormSection::Name.title(), "Skill Name");
        assert_eq!(FormSection::Description.title(), "Description");
        assert_eq!(FormSection::Preview.title(), "Preview & Save");
    }

    #[test]
    fn test_form_section_index() {
        assert_eq!(FormSection::Name.index(), 0);
        assert_eq!(FormSection::Description.index(), 1);
        assert_eq!(FormSection::Preview.index(), 8);
    }

    #[test]
    fn test_form_section_all() {
        let all = FormSection::all();
        assert_eq!(all.len(), 9);
        assert_eq!(all[0], FormSection::Name);
        assert_eq!(all[8], FormSection::Preview);
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.current_section, FormSection::Name);
        assert_eq!(app.text_areas.len(), 8);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_app_progress() {
        let app = App::new();
        let progress = app.progress();
        assert!((progress - 1.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_skill_from_output() {
        let output = r#"Some thinking text
## Skill: test-skill

### Description

A test skill

### Context

- Rust

### Tags

#test
more text"#;

        let skill = App::extract_skill_from_output(output);
        assert!(skill.starts_with("## Skill: test-skill"));
        assert!(skill.contains("### Description"));
        assert!(skill.contains("A test skill"));
    }

    #[test]
    fn test_extract_skill_with_duplicates() {
        let output = r#"## Skill: test

First skill content

## Skill: test

Second skill content"#;

        let skill = App::extract_skill_from_output(output);
        assert!(skill.contains("First skill content"));
        assert!(!skill.contains("Second skill content"));
    }

    #[test]
    fn test_parse_enhanced_skill_complete() {
        let mut app = App::new();
        let skill_markdown = r#"## Skill: database-optimizer

### Description

Optimizes database queries and performance by analyzing execution plans and suggesting improvements.

### Context

- SQL databases
- Query optimization
- Database performance tuning

### Inputs

- **query**: The SQL query to optimize
- **database_type**: The type of database (PostgreSQL, MySQL, etc.)
- **current_performance**: Current query execution metrics

### Steps

1. Analyze the provided SQL query for inefficiencies
2. Check for missing indexes or poor join operations
3. Identify bottlenecks in the execution plan
4. Suggest optimized query alternatives
5. Recommend indexing strategies

### Output

Optimized query with explanations of changes and performance improvements.

### Constraints

- Must preserve original query functionality
- Optimization must be compatible with the database version

### Tags

#optimization #database #sql #performance"#;

        app.parse_enhanced_skill(skill_markdown);

        assert_eq!(app.skill.name, "database-optimizer");
        assert!(app.skill.description.contains("Optimizes database queries"));
        assert_eq!(app.skill.context.len(), 3);
        assert!(app.skill.context.contains(&"SQL databases".to_string()));
        assert_eq!(app.skill.inputs.len(), 3);
        assert_eq!(app.skill.inputs[0].name, "query");
        assert!(app.skill.inputs[0].description.contains("SQL query"));
        assert_eq!(app.skill.steps.len(), 5);
        assert!(app.skill.steps[0].contains("Analyze"));
        assert!(app.skill.output.contains("Optimized query"));
        assert_eq!(app.skill.constraints.len(), 2);
        assert_eq!(app.skill.tags.len(), 4);
        assert!(app.skill.tags.contains(&"optimization".to_string()));
    }

    #[test]
    fn test_parse_enhanced_skill_with_minimal_input() {
        let mut app = App::new();
        let skill_markdown = r#"## Skill: test-skill

### Description

A test skill description.

### Context

- Context 1

### Inputs

- **input1**: Description of input

### Steps

1. Step 1
2. Step 2

### Output

Test output

### Constraints

- Constraint 1

### Tags

#test #skill"#;

        app.parse_enhanced_skill(skill_markdown);

        assert_eq!(app.skill.name, "test-skill");
        assert!(!app.skill.description.is_empty());
        assert!(!app.skill.context.is_empty());
        assert!(!app.skill.inputs.is_empty());
        assert!(!app.skill.steps.is_empty());
        assert!(!app.skill.output.is_empty());
        assert!(!app.skill.constraints.is_empty());
        assert!(!app.skill.tags.is_empty());
    }

    #[test]
    fn test_parse_fill_empty_fields_preserves_name() {
        let mut app = App::new();
        // This is what AI should return after filling empty fields for "qa-senior-playwright"
        let skill_markdown = r#"## Skill: qa-senior-playwright

### Description

A comprehensive skill for senior QA engineers using Playwright to develop, maintain and execute automated test suites.

### Context

- Playwright
- JavaScript/TypeScript
- CI/CD pipelines

### Inputs

- **test-file-path**: Path to the test files
- **environment**: The environment to test against

### Steps

1. Set up the Playwright environment
2. Create test cases using Playwright selectors
3. Run tests in parallel across browsers
4. Generate HTML reports
5. Integrate with CI/CD pipeline

### Output

HTML test report with pass/fail results and screenshots

### Constraints

- Tests must be deterministic and independent
- Timeout must be set appropriately for slow networks
- Browser compatibility must be tested

### Tags

#qa #playwright #automation #testing"#;

        app.parse_enhanced_skill(skill_markdown);

        // CRITICAL: Verify the name is preserved exactly
        assert_eq!(
            app.skill.name, "qa-senior-playwright",
            "Skill name must be preserved"
        );
        assert_eq!(app.skill.description, "A comprehensive skill for senior QA engineers using Playwright to develop, maintain and execute automated test suites.", "Description must be filled");
        assert!(app.skill.context.len() > 0, "Context must be filled");
        assert!(app.skill.inputs.len() > 0, "Inputs must be filled");
        assert!(app.skill.steps.len() > 0, "Steps must be filled");
        assert!(!app.skill.output.is_empty(), "Output must be filled");
        assert!(
            app.skill.constraints.len() > 0,
            "Constraints must be filled"
        );
        assert!(app.skill.tags.len() > 0, "Tags must be filled");
    }
}
