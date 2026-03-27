# AI Skill Generator CLI

A beautiful TUI (Terminal User Interface) application built with [Ratatui](https://ratatui.rs/) for creating structured, reusable AI skills in Markdown format.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Multi-step Form**: Guided wizard-style interface for creating skills
- **Live Preview**: See your Markdown output as you type
- **Syntax Highlighting**: Color-coded preview for better readability  
- **Progress Tracking**: Visual progress bar showing completion status
- **Keyboard Navigation**: Intuitive keyboard shortcuts
- **Auto-formatting**: Automatic kebab-case naming and tag formatting
- **Instant Save**: Export to Markdown file with Ctrl+S

## Installation

### Prerequisites

- Rust 1.70 or higher

### Build from Source

```bash
git clone <repo-url>
cd ai-skill-generator
cargo build --release
```

The binary will be available at `target/release/ai-skill-generator`

### Run Directly

```bash
cargo run --release
```

## Usage

### Navigation

| Key | Action |
|-----|--------|
| `Tab` / `→` | Next section |
| `Shift+Tab` / `←` | Previous section |
| `Enter` | New line (in text areas) |
| `↑` / `↓` | Scroll (in preview) |
| `Ctrl+S` | Save skill to file |
| `Esc` / `q` | Quit |

### Sections

1. **Skill Name**: Enter a descriptive name (auto-converted to kebab-case)
2. **Description**: Explain what the skill does
3. **Context**: List technologies/frameworks (one per line)
4. **Inputs**: Define parameters as `name: description` (one per line)
5. **Steps**: List actionable steps (one per line)
6. **Output**: Describe what the skill produces
7. **Constraints**: List limitations (one per line)
8. **Tags**: Add tags (space/comma separated)
9. **Preview**: Review and save

## Output Format

Skills are saved as Markdown files with the following structure:

```markdown
## Skill: skill-name

### Description

Clear explanation of the skill...

### Context

- Technology 1
- Technology 2

### Inputs

- **input_name**: description

### Steps

1. Step one
2. Step two

### Output

What the skill produces

### Constraints

- Constraint 1

### Tags

#tag1 #tag2 #tag3
```

## Example

Creating a skill for generating React forms:

1. **Name**: `React Form Generator`
2. **Description**: `Generates reusable React forms with validation using React Hook Form and Zod`
3. **Context**:
   ```
   React
   TypeScript
   React Hook Form
   Zod
   ```
4. **Inputs**:
   ```
   fields: List of form fields and types
   validation_rules: Zod schema definitions
   ```
5. **Steps**:
   ```
   Define schema using Zod
   Initialize React Hook Form
   Bind fields to form state
   Apply validation rules
   Handle submission
   ```
6. **Output**: `Reusable React form component with validation`
7. **Constraints**:
   ```
   Clean code
   Reusable
   Scalable
   ```
8. **Tags**: `react forms validation typescript`

Press `Ctrl+S` to save as `react-form-generator.md`

## License

MIT
