use crate::app::ui_components::Paragraph;
use opencode_sdk::models::{Part, TextPart, ToolPart, FilePart, StepStartPart, StepFinishPart, SnapshotPart, ToolState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Widget},
};

#[derive(Debug, Clone)]
pub struct MessagePart<'a> {
    part: &'a Part,
}

impl<'a> MessagePart<'a> {
    pub fn new(part: &'a Part) -> Self {
        Self { part }
    }

    fn render_text_part(&self, text_part: &TextPart) -> Text<'static> {
        let content = if text_part.synthetic.unwrap_or(false) {
            format!("[Synthetic] {}", text_part.text)
        } else {
            text_part.text.clone()
        };
        
        Text::from(content)
    }

    fn render_tool_part(&self, tool_part: &ToolPart) -> Text<'static> {
        let tool_name = &tool_part.tool;
        let call_id = &tool_part.call_id;
        
        let status_text = match &*tool_part.state {
            ToolState::Pending(_) => "⏳ Pending",
            ToolState::Running(_) => "🔄 Running",
            ToolState::Completed(_) => "✅ Completed", 
            ToolState::Error(_) => "❌ Error",
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("🔧 Tool: ", Style::default().fg(Color::Cyan)),
                Span::styled(tool_name.clone(), Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::styled("   ID: ", Style::default().fg(Color::Gray)),
                Span::styled(call_id.clone(), Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("   Status: ", Style::default().fg(Color::Gray)),
                Span::styled(status_text, Style::default().fg(self.get_status_color(&*tool_part.state))),
            ]),
        ];

        // Add additional details based on state
        let mut all_lines = lines;
        match &*tool_part.state {
            ToolState::Completed(completed) => {
                let output = &completed.output;
                all_lines.push(Line::from(vec![
                    Span::styled("   Output: ", Style::default().fg(Color::Gray)),
                ]));
                // Truncate long outputs for display
                let display_output = if output.len() > 200 {
                    format!("{}...", &output[..200])
                } else {
                    output.clone()
                };
                all_lines.push(Line::from(vec![
                    Span::styled(format!("   {}", display_output), Style::default().fg(Color::White)),
                ]));
            },
            ToolState::Error(error) => {
                let error_message = &error.error;
                all_lines.push(Line::from(vec![
                    Span::styled("   Error: ", Style::default().fg(Color::Red)),
                    Span::styled(error_message.clone(), Style::default().fg(Color::Red)),
                ]));
            },
            _ => {}
        }

        Text::from(all_lines)
    }

    fn render_file_part(&self, file_part: &FilePart) -> Text<'static> {
        let file_info = if let Some(filename) = &file_part.filename {
            filename.clone()
        } else {
            file_part.url.clone()
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("📄 File: ", Style::default().fg(Color::Green)),
                Span::styled(file_info, Style::default().fg(Color::White).bold()),
            ]),
        ];

        Text::from(lines)
    }

    fn render_step_start_part(&self, _step_part: &StepStartPart) -> Text<'static> {
        Text::from(Line::from(vec![
            Span::styled("▶️ ", Style::default().fg(Color::Blue)),
            Span::styled("Step started", Style::default().fg(Color::Blue).italic()),
        ]))
    }

    fn render_step_finish_part(&self, _step_part: &StepFinishPart) -> Text<'static> {
        Text::from(Line::from(vec![
            Span::styled("⏹️ ", Style::default().fg(Color::Blue)),
            Span::styled("Step finished", Style::default().fg(Color::Blue).italic()),
        ]))
    }

    fn render_snapshot_part(&self, snapshot_part: &SnapshotPart) -> Text<'static> {
        Text::from(Line::from(vec![
            Span::styled("📸 Snapshot: ", Style::default().fg(Color::Magenta)),
            Span::styled(snapshot_part.snapshot.clone(), Style::default().fg(Color::White).bold()),
        ]))
    }

    fn get_status_color(&self, state: &ToolState) -> Color {
        match state {
            ToolState::Pending(_) => Color::Yellow,
            ToolState::Running(_) => Color::Blue,
            ToolState::Completed(_) => Color::Green,
            ToolState::Error(_) => Color::Red,
        }
    }

    pub fn to_text(&self) -> Text<'static> {
        match self.part {
            Part::Text(text_part) => self.render_text_part(text_part),
            Part::Tool(tool_part) => self.render_tool_part(tool_part),
            Part::File(file_part) => self.render_file_part(file_part),
            Part::StepStart(step_part) => self.render_step_start_part(step_part),
            Part::StepFinish(step_part) => self.render_step_finish_part(step_part),
            Part::Snapshot(snapshot_part) => self.render_snapshot_part(snapshot_part),
        }
    }

    pub fn height(&self) -> u16 {
        match self.part {
            Part::Text(text_part) => {
                // Count newlines in text content
                let line_count = text_part.text.lines().count().max(1);
                line_count as u16
            },
            Part::Tool(_) => 4, // Tool parts typically take 3-4 lines
            Part::File(_) => 1,
            Part::StepStart(_) => 1,
            Part::StepFinish(_) => 1,
            Part::Snapshot(_) => 1,
        }
    }
}

impl<'a> Widget for MessagePart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.to_text();
        let paragraph = Paragraph::new(text);
        paragraph.render(area, buf);
    }
}