pub struct TextWrapper {
    width: u16,
    tolerance: usize,
}

impl TextWrapper {
    pub fn new(width: u16, tolerance: Option<usize>) -> Self {
        let tolerance = tolerance.unwrap_or(5);
        Self { width, tolerance }
    }

    pub fn wrap_text(&self, text: &str) -> Vec<String> {
        let mut wrapped_lines = Vec::new();

        for original_line in text.lines() {
            if original_line.trim().is_empty() {
                wrapped_lines.push(String::new());
                continue;
            }

            let mut char_start = 0;
            let char_count = original_line.chars().count();

            while char_start < char_count {
                let char_end = self.find_char_split_point(original_line, char_start);

                // Convert char indices to byte indices for slicing
                let byte_start = original_line
                    .char_indices()
                    .nth(char_start)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let byte_end = if char_end < char_count {
                    original_line
                        .char_indices()
                        .nth(char_end)
                        .map(|(i, _)| i)
                        .unwrap_or(original_line.len())
                } else {
                    original_line.len()
                };

                wrapped_lines.push(original_line[byte_start..byte_end].trim_end().to_string());
                char_start = char_end;

                // Skip whitespace at start of next line
                while char_start < char_count
                    && original_line
                        .chars()
                        .nth(char_start)
                        .map_or(false, |c| c.is_whitespace())
                {
                    char_start += 1;
                }
            }
        }

        wrapped_lines
    }

    pub fn wrap_ratatui_line(&self, line: &ratatui::text::Line) -> Vec<String> {
        // Convert ratatui Line to plain text string
        let line_text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();

        self.wrap_text(&line_text)
    }

    fn find_char_split_point(&self, line: &str, char_start: usize) -> usize {
        let char_count = line.chars().count();
        let width = self.width as usize;
        let remaining_chars = char_count - char_start;

        if remaining_chars <= width {
            return char_count; // Fits entirely
        }

        let ideal_char_end = char_start + width;
        let tolerance_char_start = ideal_char_end.saturating_sub(self.tolerance);

        // Look for last whitespace within tolerance window
        for char_i in (tolerance_char_start..ideal_char_end).rev() {
            if char_i < char_count
                && line
                    .chars()
                    .nth(char_i)
                    .map_or(false, |c| c.is_whitespace())
            {
                return char_i;
            }
        }

        // No whitespace found, split mid-word at width
        ideal_char_end.min(char_count)
    }
}
