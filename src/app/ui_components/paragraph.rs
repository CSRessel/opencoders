use crate::app::ui_components::block::Block;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::Text,
    widgets::{Paragraph as RatatuiParagraph, Widget, Wrap},
};

pub struct Paragraph<'a> {
    inner: RatatuiParagraph<'a>,
}

impl<'a> Paragraph<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            inner: RatatuiParagraph::new(text),
        }
    }

    // Delegate methods to the inner Paragraph
    pub fn block(mut self, block: Block) -> Self {
        self.inner = self.inner.block(block.into_inner());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.inner = self.inner.wrap(wrap);
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.inner = self.inner.alignment(alignment);
        self
    }

    pub fn scroll(mut self, offset: (u16, u16)) -> Self {
        self.inner = self.inner.scroll(offset);
        self
    }

    pub fn line_count(self, width: u16) -> usize {
        self.inner.line_count(width)
    }

    // Convert to the inner ratatui Paragraph for rendering
    pub fn into_inner(self) -> RatatuiParagraph<'a> {
        self.inner
    }
}

impl<'a> Widget for Paragraph<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf);
    }
}

impl<'a> From<Paragraph<'a>> for RatatuiParagraph<'a> {
    fn from(paragraph: Paragraph<'a>) -> Self {
        paragraph.inner
    }
}

// Allow cloning if the inner paragraph can be cloned
impl<'a> Clone for Paragraph<'a>
where
    RatatuiParagraph<'a>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
