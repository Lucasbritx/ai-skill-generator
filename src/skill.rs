/// Represents a complete AI Skill definition
#[derive(Debug, Clone, Default)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub context: Vec<String>,
    pub inputs: Vec<Input>,
    pub steps: Vec<String>,
    pub output: String,
    pub constraints: Vec<String>,
    pub tags: Vec<String>,
}

/// Represents an input parameter for a skill
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub name: String,
    pub description: String,
}

impl Skill {
    /// Convert skill name to kebab-case
    pub fn kebab_case_name(&self) -> String {
        self.name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Generate Markdown representation of the skill
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Skill name header
        let skill_name = if self.name.is_empty() {
            "untitled-skill".to_string()
        } else {
            self.kebab_case_name()
        };
        md.push_str(&format!("## Skill: {}\n\n", skill_name));

        // Description
        if !self.description.is_empty() {
            md.push_str("### Description\n\n");
            md.push_str(&self.description);
            md.push_str("\n\n");
        }

        // Context
        if !self.context.is_empty() && self.context.iter().any(|c| !c.is_empty()) {
            md.push_str("### Context\n\n");
            for ctx in &self.context {
                if !ctx.is_empty() {
                    md.push_str(&format!("- {}\n", ctx));
                }
            }
            md.push('\n');
        }

        // Inputs
        if !self.inputs.is_empty() && self.inputs.iter().any(|i| !i.name.is_empty()) {
            md.push_str("### Inputs\n\n");
            for input in &self.inputs {
                if !input.name.is_empty() {
                    if input.description.is_empty() {
                        md.push_str(&format!("- **{}**\n", input.name));
                    } else {
                        md.push_str(&format!("- **{}**: {}\n", input.name, input.description));
                    }
                }
            }
            md.push('\n');
        }

        // Steps
        if !self.steps.is_empty() && self.steps.iter().any(|s| !s.is_empty()) {
            md.push_str("### Steps\n\n");
            for (i, step) in self.steps.iter().enumerate() {
                if !step.is_empty() {
                    md.push_str(&format!("{}. {}\n", i + 1, step));
                }
            }
            md.push('\n');
        }

        // Output
        if !self.output.is_empty() {
            md.push_str("### Output\n\n");
            md.push_str(&self.output);
            md.push_str("\n\n");
        }

        // Constraints
        if !self.constraints.is_empty() && self.constraints.iter().any(|c| !c.is_empty()) {
            md.push_str("### Constraints\n\n");
            for constraint in &self.constraints {
                if !constraint.is_empty() {
                    md.push_str(&format!("- {}\n", constraint));
                }
            }
            md.push('\n');
        }

        // Tags
        if !self.tags.is_empty() && self.tags.iter().any(|t| !t.is_empty()) {
            md.push_str("### Tags\n\n");
            let tags: Vec<String> = self
                .tags
                .iter()
                .filter(|t| !t.is_empty())
                .map(|t| {
                    let tag = t.trim_start_matches('#');
                    format!("#{}", tag)
                })
                .collect();
            md.push_str(&tags.join(" "));
            md.push('\n');
        }

        md
    }

    /// Check if the skill has minimum required fields
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.description.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kebab_case() {
        let skill = Skill {
            name: "React Form Generator".to_string(),
            ..Default::default()
        };
        assert_eq!(skill.kebab_case_name(), "react-form-generator");
    }

    #[test]
    fn test_to_markdown() {
        let skill = Skill {
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            context: vec!["Rust".to_string(), "CLI".to_string()],
            inputs: vec![Input {
                name: "input1".to_string(),
                description: "First input".to_string(),
            }],
            steps: vec!["Step one".to_string(), "Step two".to_string()],
            output: "Test output".to_string(),
            constraints: vec!["Must be fast".to_string()],
            tags: vec!["rust".to_string(), "cli".to_string()],
        };

        let md = skill.to_markdown();
        assert!(md.contains("## Skill: test-skill"));
        assert!(md.contains("### Description"));
        assert!(md.contains("A test skill"));
    }
}
