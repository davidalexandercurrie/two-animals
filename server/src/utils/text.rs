/// Wrap text to fit within a specified width, preserving word boundaries
pub fn wrap_text(text: &str, width: usize, indent: &str) -> String {
    if text.len() <= width {
        return format!("{}{}", indent, text);
    }
    
    let mut result = Vec::new();
    let mut current_line = String::new();
    let effective_width = width.saturating_sub(indent.len());
    
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= effective_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(format!("{}{}", indent, current_line));
            current_line = word.to_string();
        }
    }
    
    if !current_line.is_empty() {
        result.push(format!("{}{}", indent, current_line));
    }
    
    result.join("\n")
}