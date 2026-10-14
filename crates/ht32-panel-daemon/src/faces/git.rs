//! Git status face showing a branch and a colored commit graph.

use super::{Complication, EnabledComplications, Face, Theme};
use crate::rendering::Canvas;
use crate::sensors::data::SystemData;

const GRAPH_COLORS: [u32; 5] = [0x7EE787, 0x79C0FF, 0xD2A8FF, 0xFFA657, 0xFF7B72];

pub struct GitFace;

impl Face for GitFace {
    fn name(&self) -> &str {
        "git"
    }

    fn available_complications(&self) -> Vec<Complication> {
        Vec::new()
    }

    fn render(
        &self,
        canvas: &mut Canvas,
        data: &SystemData,
        theme: &Theme,
        _complications: &EnabledComplications,
    ) {
        let (width, height) = canvas.dimensions();
        let portrait = width < 200;
        let margin = 8;
        let graph_x = margin + 3;
        let font_size = if portrait { 16.0 } else { 20.0 };
        let small_size = if portrait { 9.0 } else { 10.0 };
        let commit_size = if portrait { 11.0 } else { 12.0 };
        let header_height = if portrait { 35 } else { 40 };
        let commit_line_height = canvas.line_height(commit_size).max(8);
        let graph_color = GRAPH_COLORS[0];

        // Branch header, styled like a compact source-control panel.
        canvas.fill_rect(
            margin,
            6,
            width.saturating_sub((margin * 2) as u32),
            (header_height - 5) as u32,
            0x18232D,
        );
        canvas.draw_line(graph_x, 11, graph_x, 26, 2.0, graph_color);
        canvas.fill_circle(graph_x, 13, 3, graph_color);
        canvas.draw_text(margin + 11, 7, "BRANCH", small_size, 0x8B949E);
        canvas.draw_text(
            margin + 11,
            17,
            data.git_branch
                .as_deref()
                .unwrap_or("detached / unavailable"),
            font_size,
            graph_color,
        );

        let mut y = header_height + 2;
        if data.git_commits.is_empty() {
            canvas.draw_line(
                graph_x,
                header_height,
                graph_x,
                height as i32 - margin,
                2.0,
                graph_color,
            );
            canvas.fill_circle(graph_x, y + 4, 3, graph_color);
            canvas.draw_text(margin + 12, y, "No recent commits", font_size, theme.text);
            return;
        }

        let commits: Vec<_> = data.git_commits.iter().take(5).collect();
        let hash_x = margin + 12;
        let mut entries = Vec::with_capacity(commits.len());
        for (index, commit) in commits.iter().enumerate() {
            let hash_width = canvas.text_width(&commit.hash, small_size);
            let subject_x = hash_x + hash_width + 5;
            let first_subject_width = (width as i32 - margin - subject_x).max(8);
            let remaining_width = (width as i32 - margin - hash_x).max(8);
            let subject_lines = wrap_text(
                &commit.subject,
                canvas,
                commit_size,
                first_subject_width,
                remaining_width,
            );
            let entry_height =
                (subject_lines.len() as i32 * commit_line_height).max(commit_line_height);
            entries.push((index, subject_lines, entry_height));
        }

        let available_height = height as i32 - margin - y;
        let total_height: i32 = entries
            .iter()
            .map(|(_, _, entry_height)| *entry_height)
            .sum();
        let start = if total_height > available_height && !entries.is_empty() {
            (data.timestamp / 4) as usize % entries.len()
        } else {
            0
        };

        for offset in 0..entries.len() {
            let (index, subject_lines, entry_height) = &entries[(start + offset) % entries.len()];
            if y + entry_height > height as i32 - margin {
                break;
            }
            let commit = commits[*index];
            let color = GRAPH_COLORS[*index % GRAPH_COLORS.len()];
            let hash_width = canvas.text_width(&commit.hash, small_size);
            let subject_x = hash_x + hash_width + 5;

            // The rail and nodes give each commit its own GitLens-like color.
            let node_y = y + commit_line_height / 2;
            canvas.draw_line(graph_x, node_y, graph_x, y + entry_height, 1.5, color);
            canvas.fill_circle(graph_x, node_y, 3, color);
            canvas.draw_text(hash_x, y, &commit.hash, small_size, color);

            for (line_index, line) in subject_lines.iter().enumerate() {
                let line_x = if line_index == 0 { subject_x } else { hash_x };
                let line_y = y + line_index as i32 * commit_line_height;
                canvas.draw_text(line_x, line_y, line, commit_size, theme.text);
            }
            y += entry_height;
        }
    }
}

/// Wrap commit subjects to the available width. The first line shares space
/// with the hash; following lines use the full content width.
fn wrap_text(
    text: &str,
    canvas: &Canvas,
    size: f32,
    first_width: i32,
    full_width: i32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut remaining = text.trim();
    let mut max_width = first_width;

    while !remaining.is_empty() {
        let mut end = 0;
        let mut last_space = None;
        for (offset, ch) in remaining.char_indices() {
            let next_end = offset + ch.len_utf8();
            if canvas.text_width(&remaining[..next_end], size) > max_width {
                break;
            }
            end = next_end;
            if ch.is_whitespace() {
                last_space = Some(next_end);
            }
        }

        if end == 0 {
            // Handle a single very long word by breaking it at character boundaries.
            let mut last_fitting = 0;
            for (offset, ch) in remaining.char_indices() {
                let next_end = offset + ch.len_utf8();
                if canvas.text_width(&remaining[..next_end], size) > max_width {
                    break;
                }
                last_fitting = next_end;
            }
            end = if last_fitting == 0 {
                remaining.chars().next().map(char::len_utf8).unwrap_or(0)
            } else {
                last_fitting
            };
        } else if end < remaining.len() {
            if let Some(space) = last_space.filter(|space| *space < end) {
                end = space;
            }
        }

        let line = remaining[..end].trim_end();
        if !line.is_empty() {
            lines.push(line.to_string());
        }
        remaining = remaining[end..].trim_start();
        max_width = full_width;
    }

    if lines.is_empty() {
        lines.push(String::from("(no subject)"));
    }
    lines
}
