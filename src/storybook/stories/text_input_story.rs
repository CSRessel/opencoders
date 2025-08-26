use crate::app::ui_components::TextInputArea;
use crate::storybook::MockModel;
use ratatui::{layout::Rect, widgets::Widget};

pub struct TextInputStory {
    pub component: TextInputArea,
    pub mock_model: MockModel,
    pub variants: Vec<StoryVariant>,
}

#[derive(Debug, Clone)]
pub enum StoryVariant {
    Empty,
    WithPlaceholder,
    Focused,
    WithContent,
}

impl TextInputStory {
    pub fn new() -> Self {
        let mock_model = MockModel::new();

        Self {
            component: TextInputArea::with_placeholder("Storybook Demo - Type here..."),
            mock_model,
            variants: vec![
                StoryVariant::Empty,
                StoryVariant::WithPlaceholder,
                StoryVariant::Focused,
                StoryVariant::WithContent,
            ],
        }
    }

    pub fn render_variant(
        &mut self,
        variant: &StoryVariant,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        match variant {
            StoryVariant::Empty => {
                let component = TextInputArea::new();
                component.render(area, buf);
            }
            StoryVariant::WithPlaceholder => {
                let component = TextInputArea::with_placeholder("Enter your message...");
                component.render(area, buf);
            }
            StoryVariant::Focused => {
                let mut component = TextInputArea::with_placeholder("Focused input");
                component.set_focus(true);
                component.render(area, buf);
            }
            StoryVariant::WithContent => {
                // Mock some side effects
                self.mock_model.mock_submit_message("Sample message");
                let component = &self.component;
                component.render(area, buf);
            }
        }
    }

    pub fn get_variant_name(&self, variant: &StoryVariant) -> &'static str {
        match variant {
            StoryVariant::Empty => "Empty",
            StoryVariant::WithPlaceholder => "With Placeholder",
            StoryVariant::Focused => "Focused",
            StoryVariant::WithContent => "With Content",
        }
    }
}

