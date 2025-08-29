use crate::app::tea_model::AttachedFile;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

#[derive(Debug, Clone)]
pub struct AttachmentDisplay {
    pub files: Vec<AttachedFile>,
}

impl AttachmentDisplay {
    pub fn new(files: Vec<AttachedFile>) -> Self {
        Self { files }
    }

    /// Render as a simple inline indicator (e.g., "📎 3 files")
    pub fn render_inline(&self, area: Rect, buf: &mut Buffer) {
        if !self.files.is_empty() {
            let attachment_text = if self.files.len() == 1 {
                format!("📎 {} file", self.files.len())
            } else {
                format!("📎 {} files", self.files.len())
            };
            
            let span = Span::styled(
                attachment_text,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
            );
            
            let line = Line::from(vec![span]);
            line.render(area, buf);
        }
    }

    /// Render detailed view showing all attached files
    pub fn render_detailed(&self, area: Rect, buf: &mut Buffer) {
        if self.files.is_empty() {
            return;
        }

        let items: Vec<ListItem> = self
            .files
            .iter()
            .map(|file| {
                let spans = vec![
                    Span::styled("📎 ", Style::default().fg(Color::Cyan)),
                    Span::styled(&file.display_name, Style::default().fg(Color::White)),
                ];
                ListItem::new(Line::from(spans))
            })
            .collect();

        let title = if self.files.len() == 1 {
            "Attached File"
        } else {
            "Attached Files"
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));

        list.render(area, buf);
    }

    /// Check if there are any attachments
    pub fn has_attachments(&self) -> bool {
        !self.files.is_empty()
    }

    /// Get count of attachments
    pub fn count(&self) -> usize {
        self.files.len()
    }
}

impl Widget for &AttachmentDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_detailed(area, buf);
    }
}