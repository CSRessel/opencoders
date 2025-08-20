use crate::app::ui_components::Paragraph;
use opencode_sdk::models::{
    FilePart, GetSessionByIdMessage200ResponseInner, Part, SnapshotPart, StepFinishPart,
    StepStartPart, TextPart, ToolPart, ToolState,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::Widget,
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageContext {
    Inline,     // For tea_view.rs manual printing
    Fullscreen, // For message_log.rs
}

#[derive(Debug, Clone)]
pub struct MessageRenderer {
    parts: Vec<Part>,
    context: MessageContext,
    expanded_tools: HashSet<String>, // Track which tools are expanded (fullscreen only)
}

#[derive(Debug, Clone)]
struct StepGroup {
    text_parts: Vec<TextPart>,
    tool_parts: Vec<ToolPart>,
    file_parts: Vec<FilePart>,
}

impl MessageRenderer {
    pub fn new(parts: Vec<Part>, context: MessageContext) -> Self {
        Self {
            parts,
            context,
            expanded_tools: HashSet::new(),
        }
    }

    pub fn from_message(
        message: &GetSessionByIdMessage200ResponseInner,
        context: MessageContext,
    ) -> Self {
        Self::new(message.parts.clone(), context)
    }

    pub fn from_message_container(
        container: &crate::app::message_state::MessageContainer,
        context: MessageContext,
    ) -> Self {
        let parts: Vec<Part> = container
            .part_order
            .iter()
            .filter_map(|part_id| container.parts.get(part_id).cloned())
            .collect();
        Self::new(parts, context)
    }

    fn get_tool_status_color(&self, state: &ToolState) -> Color {
        // TODO: Make this configurable via theme/config system
        let tool_state = match state {
            ToolState::Pending(s) => s.status.clone(),
            ToolState::Running(s) => s.status.clone(),
            ToolState::Completed(s) => s.status.clone(),
            ToolState::Error(s) => s.status.clone(),
        };
        match tool_state.as_str() {
            "pending" => Color::Yellow,
            "running" => Color::Blue,
            "completed" => Color::Green,
            "error" => Color::Red,
            _ => Color::default(),
        }
    }

    fn format_tool_args(&self, tool_name: &str, _call_id: &str) -> String {
        // TODO: Parse and format tool arguments from call data
        // For now, use placeholder formatting
        match tool_name {
            "todowrite" => "Update Todos".to_string(),
            "glob" => "pattern: \"**/*ui*\"".to_string(),
            "grep" => "pattern: \"pub const TEXT_INPUT_HEIGHT\"".to_string(),
            "read" => "src/app/ui_components/text_input.rs".to_string(),
            "bash" => "cargo check".to_string(),
            _ => "".to_string(),
        }
    }

    fn format_tool_result_summary(&self, tool_part: &ToolPart) -> String {
        match &*tool_part.state {
            ToolState::Completed(completed) => {
                let output = &completed.output;
                match tool_part.tool.as_str() {
                    "todowrite" => {
                        // TODO: Parse todo list from output and show checkbox summary
                        "Updated todo list".to_string()
                    }
                    "glob" => {
                        // TODO: Parse file count from output
                        "Found 100 files".to_string()
                    }
                    "grep" => {
                        // TODO: Parse match count from output
                        "Found 1 file".to_string()
                    }
                    "read" => {
                        // TODO: Parse line count from output
                        "Read 290 lines".to_string()
                    }
                    "bash" => {
                        if output.contains("error") {
                            "Build failed".to_string()
                        } else {
                            "Checking opencoders".to_string()
                        }
                    }
                    _ => {
                        // Generic truncated output
                        if output.len() > 50 {
                            format!("{}...", &output[..50])
                        } else {
                            output.clone()
                        }
                    }
                }
            }
            ToolState::Running(_) => "Running...".to_string(),
            ToolState::Pending(_) => "Pending...".to_string(),
            ToolState::Error(error) => format!("Error: {}", error.error),
        }
    }

    fn render_todo_list_content(&self, _tool_part: &ToolPart) -> Vec<Line<'static>> {
        // TODO: Parse actual todo list from tool output
        // For now, return placeholder todo items
        let mut lines = Vec::new();

        // Example todo items - these should be parsed from actual tool output
        let todo_items = vec![
            ("☒", "Glob for all files mentioning 'ui' in the path"),
            (
                "☐",
                "Grep for the specific file that defines `pub const TEXT_INPUT_HEIGHT`",
            ),
            ("☐", "Read the contents of that file"),
            (
                "☐",
                "Edit the file to add a comment at the top with the list of public functions",
            ),
            (
                "☐",
                "Run `cargo check` to confirm the project still compiles",
            ),
        ];

        for (checkbox, text) in todo_items {
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()), // 5-space indent for todo items
                Span::styled(
                    checkbox,
                    Style::default().fg(if checkbox == "☒" {
                        Color::Green
                    } else {
                        Color::Gray
                    }),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(text, Style::default().fg(Color::White)),
            ]));
        }

        lines
    }

    fn render_tool_part(&self, tool_part: &ToolPart) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Status-based bullet point color
        let bullet_color = self.get_tool_status_color(&*tool_part.state);
        let tool_args = self.format_tool_args(&tool_part.tool, &tool_part.call_id);

        // Tool call header
        let tool_header = if tool_args.is_empty() {
            format!("● {}", tool_part.tool)
        } else {
            format!("● {}({})", tool_part.tool, tool_args)
        };

        lines.push(Line::from(vec![Span::styled(
            tool_header,
            Style::default().fg(bullet_color),
        )]));

        // Result summary with tree connector
        let result_summary = self.format_tool_result_summary(tool_part);
        let summary_line = match self.context {
            MessageContext::Inline => {
                format!("  ⎿  {}", result_summary)
            }
            MessageContext::Fullscreen => {
                // Only show expand option in fullscreen mode
                if self.expanded_tools.contains(&tool_part.call_id) {
                    format!("  ⎿  {} (ctrl+r to collapse)", result_summary)
                } else {
                    format!("  ⎿  {} (ctrl+r to expand)", result_summary)
                }
            }
        };

        lines.push(Line::from(vec![Span::styled(
            summary_line,
            Style::default().fg(Color::Gray),
        )]));

        // Special handling for todowrite tool - show todo list
        if tool_part.tool == "todowrite" {
            lines.extend(self.render_todo_list_content(tool_part));
        }

        // In fullscreen mode, show expanded output if requested
        if self.context == MessageContext::Fullscreen
            && self.expanded_tools.contains(&tool_part.call_id)
        {
            if let ToolState::Completed(_completed) = &*tool_part.state {
                // TODO: Implement expanded tool output rendering
                lines.push(Line::from(vec![Span::styled(
                    "    [Expanded output would go here]",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        lines
    }

    fn render_text_part(&self, text_part: &TextPart, is_grouped: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Skip synthetic text parts
        if text_part.synthetic.unwrap_or(false) {
            return lines;
        }

        let content = text_part.text.clone();

        // Determine prefix based on context
        let prefix = if is_grouped {
            "  " // 2-space indent for grouped text
        } else {
            "● " // Bullet for standalone text
        };

        // Split content into lines and apply prefix
        for line in content.lines() {
            if line.trim().is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(prefix.to_string(), Style::default().fg(Color::White)),
                    Span::styled(line.to_string(), Style::default().fg(Color::White)),
                ]));
            }
        }

        lines
    }

    fn group_parts_into_steps(&self) -> Vec<StepGroup> {
        let mut groups = Vec::new();
        let mut current_group = StepGroup {
            text_parts: Vec::new(),
            tool_parts: Vec::new(),
            file_parts: Vec::new(),
        };
        let mut in_step = false;

        for part in &self.parts {
            match part {
                Part::StepStart(_) => {
                    // Start a new step group
                    if in_step {
                        // Finish previous group
                        if !current_group.text_parts.is_empty()
                            || !current_group.tool_parts.is_empty()
                            || !current_group.file_parts.is_empty()
                        {
                            groups.push(current_group);
                        }
                    }
                    current_group = StepGroup {
                        text_parts: Vec::new(),
                        tool_parts: Vec::new(),
                        file_parts: Vec::new(),
                    };
                    in_step = true;
                }
                Part::StepFinish(_) => {
                    // Finish current step group
                    if in_step {
                        if !current_group.text_parts.is_empty()
                            || !current_group.tool_parts.is_empty()
                            || !current_group.file_parts.is_empty()
                        {
                            groups.push(current_group);
                        }
                        current_group = StepGroup {
                            text_parts: Vec::new(),
                            tool_parts: Vec::new(),
                            file_parts: Vec::new(),
                        };
                    }
                    in_step = false;
                }
                Part::Text(text_part) => {
                    current_group.text_parts.push((**text_part).clone());
                }
                Part::Tool(tool_part) => {
                    current_group.tool_parts.push((**tool_part).clone());
                }
                Part::File(file_part) => {
                    current_group.file_parts.push((**file_part).clone());
                }
                Part::Snapshot(_) => {
                    // Skip snapshot parts for now
                }
            }
        }

        // Don't forget the last group if we're still in a step
        if in_step
            && (!current_group.text_parts.is_empty()
                || !current_group.tool_parts.is_empty()
                || !current_group.file_parts.is_empty())
        {
            groups.push(current_group);
        }

        groups
    }

    pub fn render(&self) -> Text<'static> {
        let mut lines = Vec::new();
        let step_groups = self.group_parts_into_steps();

        // Handle case where there are no step groups (ungrouped parts)
        if step_groups.is_empty() {
            // Render parts individually without grouping
            for part in &self.parts {
                match part {
                    Part::Text(text_part) => {
                        lines.extend(self.render_text_part(text_part, false));
                    }
                    Part::Tool(tool_part) => {
                        lines.extend(self.render_tool_part(tool_part));
                    }
                    _ => {} // Skip other part types when ungrouped
                }
            }
        } else {
            // Render grouped parts
            for group in step_groups {
                // Render text parts first (grouped)
                for text_part in &group.text_parts {
                    lines.extend(self.render_text_part(text_part, true));
                }

                // Render tool parts
                for tool_part in &group.tool_parts {
                    lines.extend(self.render_tool_part(tool_part));
                }

                // Add spacing between groups
                lines.push(Line::from(""));
            }
        }

        Text::from(lines)
    }

    pub fn height(&self) -> u16 {
        let text = self.render();
        text.lines.len() as u16
    }
}

// Legacy MessagePart for backward compatibility
#[derive(Debug, Clone)]
pub struct MessagePart<'a> {
    part: &'a Part,
}

impl<'a> MessagePart<'a> {
    pub fn new(part: &'a Part) -> Self {
        Self { part }
    }

    pub fn to_text(&self) -> Text<'static> {
        // Use new MessageRenderer for single part
        let renderer = MessageRenderer::new(vec![self.part.clone()], MessageContext::Fullscreen);
        renderer.render()
    }

    pub fn height(&self) -> u16 {
        let text = self.to_text();
        text.lines.len() as u16
    }
}

impl<'a> Widget for MessagePart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.to_text();
        let paragraph = Paragraph::new(text);
        paragraph.render(area, buf);
    }
}

