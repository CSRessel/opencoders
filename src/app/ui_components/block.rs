use crate::app::view_model_context::ViewModelContext;
use ratatui::{
    style::{Style, Styled},
    widgets::{Block as RatatuiBlock, BorderType, Borders, Padding},
};

pub struct Block {
    inner: RatatuiBlock<'static>,
}

impl Block {
    pub fn new() -> Self {
        let model = ViewModelContext::current();
        let mut block = RatatuiBlock::new();

        // Apply default styling based on model state
        if model.ui_is_rounded() {
            block = block.border_type(BorderType::Rounded);
        } else {
            block = block.border_type(BorderType::Plain);
        }

        Self { inner: block }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn bordered() -> Self {
        let model = ViewModelContext::current();
        let mut block = RatatuiBlock::bordered();

        // Apply default styling based on model state
        if model.ui_is_rounded() {
            block = block.border_type(BorderType::Rounded);
        } else {
            block = block.border_type(BorderType::Plain);
        }

        Self { inner: block }
    }

    // Delegate methods to the inner Block
    pub fn borders(mut self, borders: Borders) -> Self {
        self.inner = self.inner.borders(borders);
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.inner = self.inner.border_type(border_type);
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.inner = self.inner.border_style(style);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<ratatui::text::Line<'static>>,
    {
        self.inner = self.inner.title(title);
        self
    }

    pub fn title_style(mut self, style: Style) -> Self {
        self.inner = self.inner.title_style(style);
        self
    }

    pub fn title_bottom<T>(mut self, title: T) -> Self
    where
        T: Into<ratatui::text::Line<'static>>,
    {
        self.inner = self.inner.title_bottom(title);
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }

    pub fn inner(self, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
        self.inner.inner(area)
    }

    // Convert to the inner ratatui Block for rendering
    pub fn into_inner(self) -> RatatuiBlock<'static> {
        self.inner
    }
}

impl From<Block> for RatatuiBlock<'static> {
    fn from(block: Block) -> Self {
        block.inner
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Styled trait to support stylize methods like .gray()
impl Styled for Block {
    type Item = Block;

    fn style(&self) -> Style {
        // RatatuiBlock doesn't have a getter for style, so we'll return a default
        // This is a limitation but should work for most use cases
        Style::default()
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.inner = self.inner.style(style);
        self
    }
}

