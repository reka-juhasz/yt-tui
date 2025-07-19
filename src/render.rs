use crate::colors::Theme;
use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn render_home<'a>(theme: &Theme) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Welcome to the YouTube TUI client!",
            Style::default().fg(theme.home_text.0), // Text color here
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .style(Style::default().fg(theme.home_box.0))
            .border_type(BorderType::Plain),
    )
}

pub fn render_playlists<'a>(
    theme: &Theme,
    playlists: &'a [(String, String)],
    playlist_selection_mode: bool,
    playlist_number_input: &'a str,
) -> Paragraph<'a> {
    let mut lines: Vec<Spans> = if playlists.is_empty() {
        vec![Spans::from(vec![Span::styled(
            "No playlists found.",
            Style::default().fg(theme.account_auth_failure.0),
        )])]
    } else {
        playlists
            .iter()
            .enumerate()
            .map(|(i, (title, id))| {
                Spans::from(vec![
                    Span::styled(
                        format!("{:02}. ", i + 1),
                        Style::default().fg(theme.playlist_number.0),
                    ),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(theme.playlist_name.0)
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
                    .fg(theme.playlist_number.0)
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
                .style(Style::default().fg(theme.playlist_box.0))
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
            .style(Style::default().fg(Color::White))
            .title("Videos")
            .border_type(BorderType::Plain),
    )
}

pub fn render_accounts<'a>(theme: &Theme, messages: &'a [String]) -> Paragraph<'a> {
    let mut lines = vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Your YouTube account info will appear here:",
            Style::default()
                .fg(theme.account_info.0)
                .add_modifier(Modifier::BOLD),
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
                .style(Style::default().fg(theme.account_box.0))
                .title("Account")
                .border_type(BorderType::Plain),
        )
}

pub fn render_commands<'a>(theme: &Theme) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Current commands:",
            Style::default().fg(theme.command_text_even.0),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "q: to quit",
            Style::default().fg(theme.command_text_odd.0),
        )]),
        Spans::from(vec![Span::styled(
            "a: to show accounts",
            Style::default().fg(theme.command_text_even.0),
        )]),
        Spans::from(vec![Span::styled(
            "c: to show this commands message",
            Style::default().fg(theme.command_text_odd.0),
        )]),
        Spans::from(vec![Span::styled(
            "h: to go to the home tab",
            Style::default().fg(theme.command_text_even.0),
        )]),
        Spans::from(vec![Span::styled(
            "p: to show playlists",
            Style::default().fg(theme.command_text_odd.0),
        )]),
        Spans::from(vec![Span::styled(
            "v: to show videos in playlists",
            Style::default().fg(theme.command_text_even.0),
        )]),
        Spans::from(vec![Span::styled(
            "s: to search yt",
            Style::default().fg(theme.command_text_odd.0),
        )]),
        Spans::from(vec![Span::styled(
            "b: to bind a playlist to be the selected playlist",
            Style::default().fg(theme.command_text_even.0),
        )]),
        Spans::from(vec![Span::styled(
            "press 'q' while playing playlists to skip the current song",
            Style::default().fg(theme.command_text_odd.0),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.command_box.0))
            .title("Commands")
            .border_type(BorderType::Plain),
    )
}

pub fn render_search<'a>(
    theme: &Theme,
    search_results: &'a [(String, String, String, String)],
    search_attempted: bool,
    search_selection_mode: bool,
    search_number_input: &'a str,
) -> Paragraph<'a> {
    let mut lines: Vec<Spans> = if !search_attempted {
        vec![Spans::from(Span::styled(
            "Type and press Enter to search...",
            Style::default().fg(theme.search_box.0),
        ))]
    } else if search_results.is_empty() {
        vec![Spans::from(Span::styled(
            "No results found.",
            Style::default().fg(theme.account_auth_failure.0),
        ))]
    } else {
        search_results
            .iter()
            .enumerate()
            .map(|(i, (title, duration_raw, uploader, _id))| {
                let duration = parse_iso8601_duration(duration_raw);

                Spans::from(vec![
                    Span::styled(
                        format!("{:02}. ", i + 1),
                        Style::default().fg(theme.search_number.0),
                    ),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(theme.search_name.0)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" by ", Style::default().fg(theme.search_uploader.0)),
                    Span::styled(uploader, Style::default().fg(theme.search_number.0)),
                    Span::styled(
                        format!(" [{}]", duration),
                        Style::default().fg(theme.search_name.0),
                    ),
                ])
            })
            .collect()
    };

    if search_selection_mode {
        lines.push(Spans::from(vec![
            Span::raw("Select video by number: "),
            Span::styled(
                search_number_input,
                Style::default()
                    .fg(theme.search_number.0)
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
                .title("Search Results")
                .style(Style::default().fg(theme.search_box.0))
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
            .style(Style::default().fg(Color::White)),
    )
}
