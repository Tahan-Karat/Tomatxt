use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Checkbox {
    pub text: String,
    pub completed: bool,
}

fn parse_checkbox_line(line: &str) -> Option<Checkbox> {
    line.trim()
        .strip_prefix('-')
        .or_else(|| line.trim().strip_prefix('*'))
        .and_then(|after_dash| after_dash.trim_start().strip_prefix('['))
        .and_then(|after_bracket| after_bracket.split_once(']'))
        .filter(|(checkbox_char, _)| checkbox_char.len() == 1)
        .and_then(|(checkbox_char, rest)| {
            checkbox_char.chars().next().map(|ch| {
                let is_completed = matches!(ch, 'x' | 'X');
                let text = rest
                    .trim_start()
                    .strip_prefix('-')
                    .map(|s| s.trim_start())
                    .unwrap_or_else(|| rest.trim_start())
                    .to_string();
                (text, is_completed)
            })
        })
        .filter(|(text, _)| !text.is_empty())
        .map(|(text, completed)| Checkbox { text, completed })
}

pub fn parse_checkboxes(content: &str) -> Vec<Checkbox> {
    content.lines().filter_map(parse_checkbox_line).collect()
}

// pub fn parse_checkboxes_with_positions(content: &str) -> Vec<(Checkbox, usize)> {
//     content
//         .lines()
//         .enumerate()
//         .filter_map(|(i, line)| parse_checkbox_line(line).map(|cb| (cb, i)))
//         .collect()
// }
//
// pub fn format_checkboxes(checkboxes: &[Checkbox]) -> String {
//     checkboxes
//         .iter()
//         .map(|cb| {
//             let checkbox_state = if cb.completed { "x" } else { " " };
//             format!("- [{}] {}", checkbox_state, cb.text)
//         })
//         .collect::<Vec<_>>()
//         .join("\n")
// }

pub fn update_checkbox_in_content(content: &str, checkbox_text: &str, new_status: bool) -> String {
    content
        .lines()
        .map(|line| {
            parse_checkbox_line(line)
                .filter(|checkbox| checkbox.text == checkbox_text)
                .map(|checkbox| {
                    let checkbox_state = if new_status { "x" } else { " " };
                    format!("- [{}] {}", checkbox_state, checkbox.text)
                })
                .unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// pub fn update_checkbox(
//     checkboxes: Vec<Checkbox>,
//     index: usize,
//     completed: bool,
// ) -> Option<Vec<Checkbox>> {
//     (index < checkboxes.len()).then(|| {
//         checkboxes
//             .into_iter()
//             .enumerate()
//             .map(|(i, mut cb)| {
//                 if i == index {
//                     cb.completed = completed;
//                 }
//                 cb
//             })
//             .collect()
//     })
// }

// Count completed checkboxes
// pub fn count_completed(checkboxes: &[Checkbox]) -> usize {
//     checkboxes.iter().filter(|cb| cb.completed).count()
// }
//
// /// Calculate progress percentage
// pub fn get_progress(checkboxes: &[Checkbox]) -> f32 {
//     if checkboxes.is_empty() {
//         return 0.0;
//     }
//     (count_completed(checkboxes) as f32 / checkboxes.len() as f32) * 100.0
// }
