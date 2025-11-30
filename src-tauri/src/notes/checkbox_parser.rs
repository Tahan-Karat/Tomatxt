use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Checkbox {
    pub text: String,
    pub completed: bool,
}

fn parse_checkbox_line(line: &str) -> Option<Checkbox> {
    let trimmed = line.trim();

    if !trimmed.starts_with('-') && !trimmed.starts_with('*') {
        return None;
    }

    let after_dash = trimmed
        .strip_prefix('-')
        .or_else(|| trimmed.strip_prefix('*'))?
        .trim_start();

    if !after_dash.starts_with('[') {
        return None;
    }

    let after_bracket = after_dash.strip_prefix('[')?;
    let (checkbox_char, rest) = after_bracket.split_once(']')?;

    if checkbox_char.len() != 1 {
        return None;
    }

    let is_completed = matches!(checkbox_char.chars().next()?, 'x' | 'X');
    let text = rest
        .trim_start()
        .strip_prefix('-')
        .and_then(|s| s.trim_start().strip_prefix(|_| true))
        .unwrap_or_else(|| rest.trim_start())
        .to_string();

    if text.is_empty() {
        return None;
    }

    Some(Checkbox {
        text,
        completed: is_completed,
    })
}

/// Parse all checkboxes from multiline content using functional approach
pub fn parse_checkboxes(content: &str) -> Vec<Checkbox> {
    content.lines().filter_map(parse_checkbox_line).collect()
}

/// Parse checkboxes with line position information
pub fn parse_checkboxes_with_positions(content: &str) -> Vec<(Checkbox, usize)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(i, line)| parse_checkbox_line(line).map(|cb| (cb, i)))
        .collect()
}

/// Convert checkboxes back to markdown format
pub fn format_checkboxes(checkboxes: &[Checkbox]) -> String {
    checkboxes
        .iter()
        .map(|cb| {
            let checkbox_state = if cb.completed { "x" } else { " " };
            format!("- [{}] {}", checkbox_state, cb.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Update checkboxes in content by replacing specific checkbox status
pub fn update_checkbox_in_content(content: &str, checkbox_text: &str, new_status: bool) -> String {
    let lines: Vec<&str> = content.lines().collect();

    lines
        .into_iter()
        .map(|line| {
            if let Some(checkbox) = parse_checkbox_line(line) {
                if checkbox.text == checkbox_text {
                    let checkbox_state = if new_status { "x" } else { " " };
                    return format!("- [{}] {}", checkbox_state, checkbox.text);
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Update checkbox completion status by index
pub fn update_checkbox(
    checkboxes: Vec<Checkbox>,
    index: usize,
    completed: bool,
) -> Option<Vec<Checkbox>> {
    if index >= checkboxes.len() {
        return None;
    }

    Some(
        checkboxes
            .into_iter()
            .enumerate()
            .map(|(i, mut cb)| {
                if i == index {
                    cb.completed = completed;
                }
                cb
            })
            .collect(),
    )
}

/// Count completed checkboxes
pub fn count_completed(checkboxes: &[Checkbox]) -> usize {
    checkboxes.iter().filter(|cb| cb.completed).count()
}

/// Calculate progress percentage
pub fn get_progress(checkboxes: &[Checkbox]) -> f32 {
    if checkboxes.is_empty() {
        return 0.0;
    }
    (count_completed(checkboxes) as f32 / checkboxes.len() as f32) * 100.0
}
