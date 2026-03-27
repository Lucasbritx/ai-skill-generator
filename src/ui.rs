use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::{App, FormSection};

/// Color palette
const PRIMARY: Color = Color::Rgb(99, 102, 241);    // Indigo
const SECONDARY: Color = Color::Rgb(168, 85, 247);  // Purple
const SUCCESS: Color = Color::Rgb(34, 197, 94);     // Green
const WARNING: Color = Color::Rgb(234, 179, 8);     // Yellow
const TEXT: Color = Color::Rgb(229, 231, 235);      // Light gray
const TEXT_DIM: Color = Color::Rgb(107, 114, 128);  // Dim gray
const BG_DARK: Color = Color::Rgb(17, 24, 39);      // Dark background

/// Render the entire UI
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: header, content, footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(2),  // Progress bar
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Footer/help
        ])
        .split(area);

    render_header(frame, main_layout[0]);
    render_progress(app, frame, main_layout[1]);
    render_content(app, frame, main_layout[2]);
    render_footer(app, frame, main_layout[3]);
}

/// Render the header with title
fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("🧠 ", Style::default()),
        Span::styled(
            "AI Skill Generator",
            Style::default()
                .fg(PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " - Create reusable AI skills",
            Style::default().fg(TEXT_DIM),
        ),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(TEXT_DIM)),
    );

    frame.render_widget(title, area);
}

/// Render progress bar showing current step
fn render_progress(app: &App, frame: &mut Frame, area: Rect) {
    let progress = app.progress();
    let label = format!(
        "Step {} of {} - {}",
        app.current_section.index() + 1,
        FormSection::all().len(),
        app.current_section.title()
    );

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(PRIMARY).bg(BG_DARK))
        .ratio(progress)
        .label(Span::styled(label, Style::default().fg(TEXT)));

    frame.render_widget(gauge, area);
}

/// Render the main content area
fn render_content(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split into sidebar and main content
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),  // Sidebar
            Constraint::Min(40),     // Main content
        ])
        .split(area);

    render_sidebar(app, frame, content_layout[0]);
    render_main_content(app, frame, content_layout[1]);
}

/// Render the sidebar with section navigation
fn render_sidebar(app: &App, frame: &mut Frame, area: Rect) {
    let sections: Vec<ListItem> = FormSection::all()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let is_current = *section == app.current_section;
            let is_completed = i < app.current_section.index();

            let indicator = if is_current {
                "▶ "
            } else if is_completed {
                "✓ "
            } else {
                "  "
            };

            let style = if is_current {
                Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
            } else if is_completed {
                Style::default().fg(SUCCESS)
            } else {
                Style::default().fg(TEXT_DIM)
            };

            ListItem::new(Line::from(vec![
                Span::raw(indicator),
                Span::styled(section.title(), style),
            ]))
        })
        .collect();

    let list = List::new(sections).block(
        Block::default()
            .title(" Sections ")
            .title_style(Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(TEXT_DIM))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(list, area);
}

/// Render the main content based on current section
fn render_main_content(app: &mut App, frame: &mut Frame, area: Rect) {
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Help text
            Constraint::Min(5),     // Input area
        ])
        .margin(1)
        .split(area);

    // Help text
    let help = Paragraph::new(app.current_section.help_text())
        .style(Style::default().fg(TEXT_DIM).italic())
        .wrap(Wrap { trim: true });
    frame.render_widget(help, inner_layout[0]);

    // Main input or preview
    if app.current_section == FormSection::Preview {
        render_preview(app, frame, inner_layout[1]);
    } else {
        render_textarea(app, frame, inner_layout[1]);
    }

    // Outer border
    let border = Block::default()
        .title(format!(" {} ", app.current_section.title()))
        .title_style(Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.editing { PRIMARY } else { TEXT_DIM }));
    frame.render_widget(border, area);
}

/// Render the text area for input
fn render_textarea(app: &mut App, frame: &mut Frame, area: Rect) {
    if let Some(textarea) = app.current_textarea() {
        textarea.set_cursor_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::REVERSED),
        );
        textarea.set_style(Style::default().fg(TEXT));

        frame.render_widget(&*textarea, area);
    }
}

/// Render the preview section with markdown output
fn render_preview(app: &App, frame: &mut Frame, area: Rect) {
    let markdown = app.skill.to_markdown();
    let lines: Vec<Line> = markdown
        .lines()
        .map(|line| {
            if line.starts_with("## ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("### ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(SECONDARY)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("- ") || line.starts_with("* ") {
                Line::from(vec![
                    Span::styled("  • ", Style::default().fg(SUCCESS)),
                    Span::styled(&line[2..], Style::default().fg(TEXT)),
                ])
            } else if line.starts_with('#') {
                // Tags
                Line::from(Span::styled(line, Style::default().fg(WARNING)))
            } else if line.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                // Numbered list
                Line::from(Span::styled(line, Style::default().fg(TEXT)))
            } else {
                Line::from(Span::styled(line, Style::default().fg(TEXT)))
            }
        })
        .collect();

    let preview = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(
            Block::default()
                .title(" Markdown Preview ")
                .title_style(Style::default().fg(SUCCESS))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SUCCESS)),
        );

    frame.render_widget(preview, area);

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state =
        ScrollbarState::new(markdown.lines().count()).position(app.preview_scroll as usize);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

/// Render the footer with keybindings
fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let keys = if app.current_section == FormSection::Preview {
        vec![
            ("↑/↓", "Scroll"),
            ("←/Tab", "Back"),
            ("Ctrl+S", "Save"),
            ("q/Esc", "Quit"),
        ]
    } else {
        vec![
            ("Tab/→", "Next"),
            ("Shift+Tab/←", "Back"),
            ("Enter", "New line"),
            ("Ctrl+S", "Save"),
            ("Esc", "Quit"),
        ]
    };

    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(TEXT_DIM)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(TEXT_DIM)),
                Span::raw("  "),
            ]
        })
        .collect();

    let mut footer_line = spans;

    // Add status message if present
    if let Some(ref msg) = app.status_message {
        footer_line.push(Span::styled(
            format!("  │  {}", msg),
            Style::default().fg(if app.saved { SUCCESS } else { WARNING }),
        ));
    }

    let footer = Paragraph::new(Line::from(footer_line))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(TEXT_DIM)),
        );

    frame.render_widget(footer, area);
}
