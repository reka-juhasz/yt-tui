use crossterm::terminal;
use std::io::{self, Write};

use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn render_home<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome to the YouTube TUI client!")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    )
}

pub fn render_playlists<'a>(
    playlists: &'a [(String, String)],
    playlist_selection_mode: bool,
    playlist_number_input: &'a str,
) -> Paragraph<'a> {
    let mut lines: Vec<Spans> = if playlists.is_empty() {
        vec![Spans::from(vec![Span::raw("No playlists found.")])]
    } else {
        playlists
            .iter()
            .enumerate()
            .map(|(i, (title, id))| {
                Spans::from(vec![
                    Span::raw(format!("{:02}. ", i + 1)),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" (ID: {})", id)),
                ])
            })
            .collect()
    };

    if playlist_selection_mode {
        lines.push(Spans::from(vec![
            Span::raw("Select playlist by number: "),
            Span::styled(
                playlist_number_input,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Playlists")
                .border_type(tui::widgets::BorderType::Plain),
        )
}

pub fn render_videos<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "Same, but for the videos in the selected playlists",
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue))
            .title("Videos")
            .border_type(BorderType::Plain),
    )
}

pub fn render_accounts<'a>(messages: &'a [String]) -> Paragraph<'a> {
    let mut lines = vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Your YouTube account info will appear here:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw("")]),
    ];

    for message in messages {
        lines.push(Spans::from(vec![Span::raw(message)]));
    }
    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false }) // ✅ this enables wrapping
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Blue))
                .title("Account")
                .border_type(BorderType::Plain),
        )
}

pub fn render_commands<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Current commands:")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("q: to quit")]),
        Spans::from(vec![Span::raw("a: to show accounts")]),
        Spans::from(vec![Span::raw("c: to show this commands message")]),
        Spans::from(vec![Span::raw("h: to go to the home tab")]),
        Spans::from(vec![Span::raw("p: to show playlists")]),
        Spans::from(vec![Span::raw("v: to show videos in playlists")]),
        Spans::from(vec![Span::raw("s: to search yt")]),
        Spans::from(vec![Span::raw(
            "b: to bind a playlist to be the selected playlist",
        )]),
        Spans::from(vec![Span::raw(
            "press 'q' while playing playlists to skip the current song",
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue))
            .title("Commands")
            .border_type(BorderType::Plain),
    )
}

pub fn render_search<'a>(
    search_results: &'a [(String, String, String, String)],
    search_attempted: bool,
) -> Paragraph<'a> {
    let lines: Vec<Spans> = if !search_attempted {
        vec![Spans::from(Span::styled(
            "Type and press Enter to search...",
            Style::default().fg(Color::DarkGray),
        ))]
    } else if search_results.is_empty() {
        vec![Spans::from(Span::styled(
            "No results found.",
            Style::default().fg(Color::Red),
        ))]
    } else {
        search_results
            .iter()
            .enumerate()
            .map(|(i, (title, duration_raw, uploader, _id))| {
                let duration = parse_iso8601_duration(duration_raw);

                Spans::from(vec![
                    Span::raw(format!("{:02}. ", i + 1)),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" by "),
                    Span::styled(uploader, Style::default().fg(Color::Magenta)),
                    Span::raw(format!(" [{}]", duration)),
                ])
            })
            .collect()
    };

    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search Results")
                .border_type(tui::widgets::BorderType::Plain),
        )
}

fn parse_iso8601_duration(duration: &str) -> String {
    let mut hours = 0;
    let mut minutes = 0;
    let mut seconds = 0;

    let mut num = String::new();
    let mut chars = duration.chars().peekable();

    while let Some(c) = chars.next() {
        if c == 'P' || c == 'T' {
            continue;
        }

        if c.is_digit(10) {
            num.push(c);
        } else {
            match c {
                'H' => {
                    if let Ok(n) = num.parse() {
                        hours = n;
                    }
                    num.clear();
                }
                'M' => {
                    if let Ok(n) = num.parse() {
                        minutes = n;
                    }
                    num.clear();
                }
                'S' => {
                    if let Ok(n) = num.parse() {
                        seconds = n;
                    }
                    num.clear();
                }
                _ => {}
            }
        }
    }

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

pub fn render_search_prompt<'a>(user_input: &'a str) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("Search YouTube:")]),
        Spans::from(vec![Span::raw(user_input)]),
    ])
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .title("Search")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow)),
    )
}
