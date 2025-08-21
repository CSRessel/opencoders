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
            if original_line.is_empty() {
                wrapped_lines.push(String::new());
                continue;
            }
            
            let mut start = 0;
            while start < original_line.len() {
                let end = self.find_split_point(original_line, start);
                wrapped_lines.push(original_line[start..end].trim_end().to_string());
                start = end;
                
                // Skip whitespace at start of next line
                while start < original_line.len() && 
                      original_line.chars().nth(start).map_or(false, |c| c.is_whitespace()) {
                    start += 1;
                }
            }
        }
        
        wrapped_lines
    }

    pub fn wrap_ratatui_line(&self, line: &ratatui::text::Line) -> Vec<String> {
        // Convert ratatui Line to plain text string
        let line_text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();
        
        self.wrap_text(&line_text)
    }

    fn find_split_point(&self, line: &str, start: usize) -> usize {
        let remaining = &line[start..];
        let width = self.width as usize;
        
        if remaining.len() <= width {
            return start + remaining.len(); // Fits entirely
        }
        
        let ideal_end = start + width;
        let tolerance_start = ideal_end.saturating_sub(self.tolerance);
        
        // Look for last whitespace within tolerance window
        for i in (tolerance_start..ideal_end).rev() {
            if line.chars().nth(i).map_or(false, |c| c.is_whitespace()) {
                return i;
            }
        }
        
        // No whitespace found, split mid-word at width
        ideal_end
    }
}