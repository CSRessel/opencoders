use crate::app::ui_components::Paragraph;
use opencode_sdk::models::{
    FilePart, Part, SessionMessages200ResponseInner, SnapshotPart, StepFinishPart, StepStartPart,
    TextPart, ToolPart, ToolState,
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

#[derive(Debug, Clone, PartialEq)]
pub enum VerbosityLevel {
    Summary, // Concise one-line descriptions
    Verbose, // Full details for all content
}

#[derive(Debug, Clone)]
pub struct MessageRenderer {
    parts: Vec<Part>,
    context: MessageContext,
    verbosity: VerbosityLevel,
    expanded_tools: HashSet<String>, // Track which tools are expanded (fullscreen only)
}

#[derive(Debug, Clone)]
struct StepGroup {
    text_parts: Vec<TextPart>,
    tool_parts: Vec<ToolPart>,
    file_parts: Vec<FilePart>,
}

impl MessageRenderer {
    pub fn new(parts: Vec<Part>, context: MessageContext, verbosity: VerbosityLevel) -> Self {
        Self {
            parts,
            context,
            verbosity,
            expanded_tools: HashSet::new(),
        }
    }

    pub fn from_message(
        message: &SessionMessages200ResponseInner,
        context: MessageContext,
        verbosity: VerbosityLevel,
    ) -> Self {
        Self::new(message.parts.clone(), context, verbosity)
    }

    pub fn from_message_container(
        container: &crate::app::message_state::MessageContainer,
        context: MessageContext,
        verbosity: VerbosityLevel,
    ) -> Self {
        let parts: Vec<Part> = container
            .part_order
            .iter()
            .filter_map(|part_id| container.parts.get(part_id).cloned())
            .collect();
        Self::new(parts, context, verbosity)
    }

    pub fn with_verbosity(mut self, verbosity: VerbosityLevel) -> Self {
        self.verbosity = verbosity;
        self
    }

    fn get_tool_status_color(&self, state: &ToolState) -> Color {
        // Check the actual status string from the API response
        match state {
            ToolState::Pending(_) => Color::Yellow,
            ToolState::Running(_) => Color::Blue,
            ToolState::Completed(_) => Color::Green,
            ToolState::Error(_) => Color::Red,
        }
    }

    fn format_tool_args(&self, tool_part: &ToolPart) -> String {
        // Parse tool arguments from state.input
        match &*tool_part.state {
            ToolState::Completed(completed) => {
                self.parse_tool_input(&tool_part.tool, &completed.input)
            }
            ToolState::Running(running) => {
                // Running state has Option<Option<Value>> input, flatten it
                if let Some(Some(input_value)) = &running.input {
                    if let Some(input_obj) = input_value.as_object() {
                        self.parse_tool_input_from_value(&tool_part.tool, input_obj)
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            ToolState::Error(error) => self.parse_tool_input(&tool_part.tool, &error.input),
            ToolState::Pending(_) => {
                // Pending state has no input field
                "".to_string()
            }
        }
    }

    fn parse_tool_input(
        &self,
        tool_name: &str,
        input: &std::collections::HashMap<String, serde_json::Value>,
    ) -> String {
        match tool_name {
            "bash" => {
                if let Some(command) = input.get("command").and_then(|v| v.as_str()) {
                    command.to_string()
                } else {
                    "".to_string()
                }
            }
            "read" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    // Show just the filename, not full path
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "write" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "patch" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "edit" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "glob" => {
                if let Some(pattern) = input.get("pattern").and_then(|v| v.as_str()) {
                    format!("pattern: \"{}\"", pattern)
                } else {
                    "".to_string()
                }
            }
            "grep" => {
                if let Some(pattern) = input.get("pattern").and_then(|v| v.as_str()) {
                    format!("pattern: \"{}\"", pattern)
                } else {
                    "".to_string()
                }
            }
            "list" => {
                if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                    if let Some(dirname) = path.split('/').last() {
                        dirname.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "todowrite" => "Update Todos".to_string(),
            "todoread" => "Read Todos".to_string(),
            "webfetch" => {
                if let Some(url) = input.get("url").and_then(|v| v.as_str()) {
                    // Show just domain for brevity using simple string parsing
                    if url.starts_with("http://") || url.starts_with("https://") {
                        if let Some(domain_start) = url.find("://").map(|i| i + 3) {
                            if let Some(path_start) = url[domain_start..].find('/') {
                                url[domain_start..domain_start + path_start].to_string()
                            } else {
                                url[domain_start..].to_string()
                            }
                        } else {
                            url.to_string()
                        }
                    } else {
                        url.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        }
    }

    fn parse_tool_input_from_value(
        &self,
        tool_name: &str,
        input: &serde_json::Map<String, serde_json::Value>,
    ) -> String {
        match tool_name {
            "bash" => {
                if let Some(command) = input.get("command").and_then(|v| v.as_str()) {
                    command.to_string()
                } else {
                    "".to_string()
                }
            }
            "read" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    // Show just the filename, not full path
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "write" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "patch" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "edit" => {
                if let Some(path) = input.get("filePath").and_then(|v| v.as_str()) {
                    if let Some(filename) = path.split('/').last() {
                        filename.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "glob" => {
                if let Some(pattern) = input.get("pattern").and_then(|v| v.as_str()) {
                    format!("pattern: \"{}\"", pattern)
                } else {
                    "".to_string()
                }
            }
            "grep" => {
                if let Some(pattern) = input.get("pattern").and_then(|v| v.as_str()) {
                    format!("pattern: \"{}\"", pattern)
                } else {
                    "".to_string()
                }
            }
            "list" => {
                if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                    if let Some(dirname) = path.split('/').last() {
                        dirname.to_string()
                    } else {
                        path.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            "todowrite" => "Update Todos".to_string(),
            "todoread" => "Read Todos".to_string(),
            "webfetch" => {
                if let Some(url) = input.get("url").and_then(|v| v.as_str()) {
                    // Show just domain for brevity using simple string parsing
                    if url.starts_with("http://") || url.starts_with("https://") {
                        if let Some(domain_start) = url.find("://").map(|i| i + 3) {
                            if let Some(path_start) = url[domain_start..].find('/') {
                                url[domain_start..domain_start + path_start].to_string()
                            } else {
                                url[domain_start..].to_string()
                            }
                        } else {
                            url.to_string()
                        }
                    } else {
                        url.to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        }
    }

    fn format_tool_result_summary(&self, tool_part: &ToolPart) -> String {
        match &*tool_part.state {
            ToolState::Completed(completed) => {
                let output = &completed.output;
                match tool_part.tool.as_str() {
                    "todowrite" => {
                        // Try to get todos from metadata first (cleaner structure)
                        if let Some(metadata_todos) = completed.metadata.get("todos") {
                            if let Some(array) = metadata_todos.as_array() {
                                format!("{} todos", array.len())
                            } else {
                                "Updated todo list".to_string()
                            }
                        } else if let Ok(todos) = serde_json::from_str::<serde_json::Value>(output)
                        {
                            // Fallback to parsing from output
                            if let Some(array) = todos.as_array() {
                                format!("{} todos", array.len())
                            } else {
                                "Updated todo list".to_string()
                            }
                        } else {
                            "Updated todo list".to_string()
                        }
                    }
                    "glob" => {
                        // Try to get count from metadata first, fallback to counting lines
                        if let Some(metadata) = completed.metadata.get("count") {
                            if let Some(count) = metadata.as_u64() {
                                format!("Found {} files", count)
                            } else {
                                let lines = output
                                    .lines()
                                    .filter(|line| !line.trim().is_empty())
                                    .count();
                                format!("Found {} files", lines)
                            }
                        } else {
                            let lines = output
                                .lines()
                                .filter(|line| !line.trim().is_empty())
                                .count();
                            if lines > 0 {
                                format!("Found {} files", lines)
                            } else {
                                "No files found".to_string()
                            }
                        }
                    }
                    "grep" => {
                        // Try to get matches count from metadata first
                        if let Some(metadata) = completed.metadata.get("matches") {
                            if let Some(matches) = metadata.as_u64() {
                                if matches > 0 {
                                    format!("Found {} matches", matches)
                                } else {
                                    "No matches found".to_string()
                                }
                            } else {
                                "Search completed".to_string()
                            }
                        } else {
                            // Fallback to parsing output
                            if output.contains("Found") && output.contains("matches") {
                                // Extract from "Found X matches" format
                                if let Some(first_line) = output.lines().next() {
                                    first_line.to_string()
                                } else {
                                    "Search completed".to_string()
                                }
                            } else {
                                let lines = output
                                    .lines()
                                    .filter(|line| !line.trim().is_empty())
                                    .count();
                                if lines > 0 {
                                    format!("Found {} matches", lines)
                                } else {
                                    "No matches found".to_string()
                                }
                            }
                        }
                    }
                    "read" => {
                        // Parse line count from read output
                        if output.starts_with("<file>") && output.contains("</file>") {
                            let line_count = output.lines().count().saturating_sub(2); // Subtract <file> and </file>
                            format!("Read {} lines", line_count)
                        } else {
                            format!("Read {} chars", output.len())
                        }
                    }
                    "write" => {
                        if output.trim().is_empty() {
                            "File written".to_string()
                        } else {
                            // Check for success indicators
                            if output.contains("successfully") || output.contains("created") {
                                "File written".to_string()
                            } else {
                                format!("Output: TODO diffs! len={}", output.len())
                            }
                        }
                    }
                    "patch" => {
                        if output.trim().is_empty() {
                            "File patched".to_string()
                        } else {
                            format!("Output: TODO diffs! len={}", output.len())
                        }
                    }
                    "edit" => {
                        if output.trim().is_empty() {
                            "File edited".to_string()
                        } else {
                            format!("Output: TODO diffs! len={}", output.len())
                        }
                    }
                    "list" => {
                        let lines = output
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .count();
                        format!("Found {} items", lines)
                    }
                    "bash" => {
                        // Check metadata for exit code first
                        if let Some(exit_code) = completed.metadata.get("exit") {
                            if let Some(code) = exit_code.as_u64() {
                                if code == 0 {
                                    if output.contains("warning") || output.contains("Warning") {
                                        "Command completed with warnings".to_string()
                                    } else if output.trim().is_empty() {
                                        "Command completed successfully".to_string()
                                    } else {
                                        // Show first meaningful line for successful commands
                                        if let Some(first_line) = output.lines().find(|line| {
                                            !line.trim().is_empty() && !line.trim().starts_with(' ')
                                        }) {
                                            self.truncate_output(first_line.trim(), 40)
                                        } else {
                                            "Command completed successfully".to_string()
                                        }
                                    }
                                } else {
                                    format!("Command failed (exit {})", code)
                                }
                            } else {
                                "Command completed".to_string()
                            }
                        } else {
                            // Fallback to output parsing
                            if output.contains("error")
                                || output.contains("Error")
                                || output.contains("ERROR")
                            {
                                "Command failed".to_string()
                            } else if output.contains("warning") || output.contains("Warning") {
                                "Command completed with warnings".to_string()
                            } else if output.trim().is_empty() {
                                "Command completed".to_string()
                            } else {
                                // Show first meaningful line
                                if let Some(first_line) =
                                    output.lines().find(|line| !line.trim().is_empty())
                                {
                                    self.truncate_output(first_line, 40)
                                } else {
                                    "Command completed".to_string()
                                }
                            }
                        }
                    }
                    "webfetch" => {
                        if output.len() > 100 {
                            format!("Fetched {} chars", output.len())
                        } else {
                            "Content fetched".to_string()
                        }
                    }
                    _ => {
                        // Generic truncated output
                        self.truncate_output(output, 50)
                    }
                }
            }
            ToolState::Running(_) => "Running...".to_string(),
            ToolState::Pending(_) => "Pending...".to_string(),
            ToolState::Error(error) => format!("Error: {}", self.truncate_output(&error.error, 40)),
        }
    }

    fn truncate_output(&self, text: &str, max_len: usize) -> String {
        if text.len() > max_len {
            format!("{}...", &text[..max_len])
        } else {
            text.to_string()
        }
    }

    fn render_todo_list_content(&self, tool_part: &ToolPart) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Parse actual todo list from tool output or metadata
        if let ToolState::Completed(completed) = &*tool_part.state {
            // Try metadata first (cleaner structure)
            let todos_source = if let Some(metadata_todos) = completed.metadata.get("todos") {
                Some(metadata_todos.clone())
            } else if let Ok(output_todos) =
                serde_json::from_str::<serde_json::Value>(&completed.output)
            {
                Some(output_todos)
            } else {
                None
            };

            if let Some(todos) = todos_source {
                if let Some(array) = todos.as_array() {
                    for todo in array {
                        if let (Some(content), Some(status)) = (
                            todo.get("content").and_then(|v| v.as_str()),
                            todo.get("status").and_then(|v| v.as_str()),
                        ) {
                            let checkbox = match status {
                                "completed" => "☒",
                                "in_progress" => "◐",
                                "cancelled" => "☒",
                                _ => "☐",
                            };

                            let checkbox_color = match status {
                                "completed" => Color::Green,
                                "in_progress" => Color::Yellow,
                                "cancelled" => Color::Red,
                                _ => Color::Gray,
                            };

                            lines.push(Line::from(vec![
                                Span::styled("     ".to_string(), Style::default()), // 5-space indent for todo items
                                Span::styled(
                                    checkbox.to_string(),
                                    Style::default().fg(checkbox_color),
                                ),
                                Span::styled(" ".to_string(), Style::default()),
                                Span::styled(
                                    content.to_string(),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                    }
                } else {
                    // Fallback: show that todos were updated but couldn't parse
                    lines.push(Line::from(vec![
                        Span::styled("     ".to_string(), Style::default()),
                        Span::styled("⎿ ".to_string(), Style::default().fg(Color::Gray)),
                        Span::styled(
                            "Todo list updated".to_string(),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));
                }
            } else {
                // Fallback for non-JSON output
                lines.push(Line::from(vec![
                    Span::styled("     ".to_string(), Style::default()),
                    Span::styled("⎿ ".to_string(), Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Todo list updated".to_string(),
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
        }

        lines
    }

    fn render_tool_part(&self, tool_part: &ToolPart) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(" "));

        // Status-based bullet point color
        let bullet_color = self.get_tool_status_color(&*tool_part.state);
        let tool_args = self.format_tool_args(tool_part);

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
        let summary_line = match (&self.context, &self.verbosity) {
            (MessageContext::Fullscreen, VerbosityLevel::Summary) => {
                format!("  ⎿  {} (ctrl+r to expand)", result_summary)
            }
            (MessageContext::Inline, _) | (_, VerbosityLevel::Verbose) => {
                format!("  ⎿  {}", result_summary)
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

        // In verbose mode, show full tool output inline
        if self.verbosity == VerbosityLevel::Verbose {
            if let ToolState::Completed(completed) = &*tool_part.state {
                lines.extend(self.render_full_tool_output(&completed.output));
            }
        }

        lines
    }

    fn render_text_part(&self, text_part: &TextPart, is_grouped: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(" "));

        // Skip synthetic text parts
        if text_part.synthetic.unwrap_or(false) {
            return lines;
        }

        let content = text_part.text.clone();

        // Determine prefix based on context
        let prefix = if is_grouped {
            "  " // 2-space indent for grouped text
        } else {
            "> " // Bullet for standalone text
        };

        // Split content into lines and apply prefix
        for line in content.lines() {
            if line.trim().is_empty() {
                lines.push(Line::from(" "));
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
                // Not properly implemented for now
                Part::Snapshot(snap_part) => current_group.text_parts.push(TextPart {
                    id: snap_part.id.clone(),
                    session_id: snap_part.session_id.clone(),
                    message_id: snap_part.message_id.clone(),
                    text: format!("TODO(snapshot) {}", snap_part.snapshot),
                    synthetic: None,
                    time: None,
                }),
                Part::Reasoning(reason_part) => current_group.text_parts.push(TextPart {
                    id: reason_part.id.clone(),
                    session_id: reason_part.session_id.clone(),
                    message_id: reason_part.message_id.clone(),
                    text: format!("TODO(reasoning) {}", reason_part.text),
                    synthetic: None,
                    time: Some(reason_part.time.clone()),
                }),
                Part::Patch(patch_part) => current_group.text_parts.push(TextPart {
                    id: patch_part.id.clone(),
                    session_id: patch_part.session_id.clone(),
                    message_id: patch_part.message_id.clone(),
                    text: format!(
                        "TODO(patch) files={}",
                        serde_json::to_string(&patch_part.files).unwrap_or("-".to_string())
                    ),
                    synthetic: None,
                    time: None,
                }),
                Part::Agent(agent_part) => current_group.text_parts.push(TextPart {
                    id: agent_part.id.clone(),
                    session_id: agent_part.session_id.clone(),
                    message_id: agent_part.message_id.clone(),
                    text: format!(
                        "TODO(agent) name={} source={}",
                        agent_part.name,
                        serde_json::to_string(&agent_part.source).unwrap_or("-".to_string())
                    ),
                    synthetic: None,
                    time: None,
                }),
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

    fn render_step_group(&self, group: &StepGroup) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Determine if this is a mixed grouping
        let has_text_parts = !group.text_parts.is_empty();
        let has_tool_parts = !group.tool_parts.is_empty();
        let is_mixed_grouping = has_text_parts && has_tool_parts;

        // Text parts visibility rules
        let show_text_parts = match self.verbosity {
            VerbosityLevel::Verbose => true,
            VerbosityLevel::Summary => !is_mixed_grouping, // Hide in mixed groupings
        };

        if show_text_parts {
            for text_part in &group.text_parts {
                lines.extend(self.render_text_part(text_part, true));
            }
        }

        // Tool parts rendering
        for tool_part in &group.tool_parts {
            lines.extend(self.render_tool_part(tool_part));
        }

        lines
    }

    fn render_full_tool_output(&self, output: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if output.trim().is_empty() {
            return lines;
        }

        // Add separator line
        lines.push(Line::from(vec![Span::styled(
            "    ┌─ Full Output:",
            Style::default().fg(Color::DarkGray),
        )]));

        // Render each line of output with proper indentation
        for line in output.lines() {
            lines.push(Line::from(vec![
                Span::styled("    │ ".to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
            ]));
        }

        // Add closing line
        lines.push(Line::from(vec![Span::styled(
            "    └─",
            Style::default().fg(Color::DarkGray),
        )]));

        lines
    }

    pub fn render(&self) -> Text<'static> {
        let mut lines = Vec::new();
        let step_groups = self.group_parts_into_steps();

        // Handle case where there are no step groups (ungrouped parts)
        if step_groups.is_empty() {
            // Create a single group from all parts to apply mixed grouping logic
            let mut ungrouped_group = StepGroup {
                text_parts: Vec::new(),
                tool_parts: Vec::new(),
                file_parts: Vec::new(),
            };
            
            for part in &self.parts {
                match part {
                    Part::Text(text_part) => {
                        ungrouped_group.text_parts.push((**text_part).clone());
                    }
                    Part::Tool(tool_part) => {
                        ungrouped_group.tool_parts.push((**tool_part).clone());
                    }
                    Part::File(file_part) => {
                        ungrouped_group.file_parts.push((**file_part).clone());
                    }
                    _ => {} // Skip other part types when ungrouped
                }
            }
            
            lines.extend(self.render_step_group(&ungrouped_group));
        } else {
            // Render grouped parts
            for group in step_groups {
                lines.extend(self.render_step_group(&group));
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
        let renderer = MessageRenderer::new(
            vec![self.part.clone()],
            MessageContext::Fullscreen,
            VerbosityLevel::Summary,
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_sdk::models::{ToolStateCompleted, ToolStateCompletedTime};
    use std::collections::HashMap;

    fn create_text_part(text: &str) -> Part {
        Part::Text(Box::new(TextPart {
            id: "text1".to_string(),
            session_id: "session1".to_string(),
            message_id: "msg1".to_string(),
            text: text.to_string(),
            synthetic: None,
            time: None,
        }))
    }

    fn create_tool_part(tool: &str, output: &str) -> Part {
        Part::Tool(Box::new(ToolPart {
            id: "tool1".to_string(),
            session_id: "session1".to_string(),
            message_id: "msg1".to_string(),
            call_id: "tool1".to_string(),
            tool: tool.to_string(),
            state: Box::new(ToolState::Completed(Box::new(ToolStateCompleted {
                input: HashMap::new(),
                output: output.to_string(),
                title: "Test Tool".to_string(),
                metadata: HashMap::new(),
                time: Box::new(ToolStateCompletedTime {
                    start: 0.0,
                    end: 1.0,
                }),
            }))),
        }))
    }

    #[test]
    fn test_summary_mode_hides_text_in_mixed_groupings() {
        let parts = vec![
            create_text_part("This is some text"),
            create_tool_part("bash", "Command output"),
        ];

        // Summary mode should hide text parts in mixed groupings
        let renderer_summary = MessageRenderer::new(
            parts.clone(),
            MessageContext::Fullscreen,
            VerbosityLevel::Summary,
        );
        let summary_text = renderer_summary.render();

        // Text should not appear in summary mode with mixed content
        let summary_content = summary_text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!summary_content.contains("This is some text"));
        assert!(summary_content.contains("bash"));

        // Verbose mode should show text parts in mixed groupings
        let renderer_verbose =
            MessageRenderer::new(parts, MessageContext::Fullscreen, VerbosityLevel::Verbose);
        let verbose_text = renderer_verbose.render();

        let verbose_content = verbose_text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(verbose_content.contains("This is some text"));
        assert!(verbose_content.contains("bash"));
    }

    #[test]
    fn test_ctrl_r_hint_only_in_fullscreen_summary() {
        let parts = vec![create_tool_part("bash", "output")];

        // Fullscreen + Summary should show ctrl+r hint
        let renderer_fs_summary = MessageRenderer::new(
            parts.clone(),
            MessageContext::Fullscreen,
            VerbosityLevel::Summary,
        );
        let text = renderer_fs_summary.render();
        let content = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(content.contains("(ctrl+r to expand)"));

        // Inline + Summary should not show ctrl+r hint
        let renderer_inline_summary = MessageRenderer::new(
            parts.clone(),
            MessageContext::Inline,
            VerbosityLevel::Summary,
        );
        let text = renderer_inline_summary.render();
        let content = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!content.contains("(ctrl+r to expand)"));

        // Fullscreen + Verbose should not show ctrl+r hint
        let renderer_fs_verbose =
            MessageRenderer::new(parts, MessageContext::Fullscreen, VerbosityLevel::Verbose);
        let text = renderer_fs_verbose.render();
        let content = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!content.contains("(ctrl+r to expand)"));
    }

    #[test]
    fn test_text_only_grouping_always_shows_text() {
        let parts = vec![create_text_part("This is standalone text")];

        // Even in summary mode, text should show when it's not mixed with tools
        let renderer_summary = MessageRenderer::new(
            parts.clone(),
            MessageContext::Fullscreen,
            VerbosityLevel::Summary,
        );
        let text = renderer_summary.render();
        let content = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(content.contains("This is standalone text"));
    }
}
