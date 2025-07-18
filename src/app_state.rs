use std::os::linux::raw::stat;

use crate::state::get_token;
use crate::utilities;
use crate::utilities::play_playlist;
use anyhow::Result;
use crossterm::event::KeyEvent;
use crossterm::{event::KeyCode, terminal::disable_raw_mode};
use oauth2::TokenResponse;

use tui::{backend::CrosstermBackend, Terminal};
pub enum Event<I> {
    Input(I),
    Tick,
    Message(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Account,
    Commands,
    Home,
    Playlists,
    Videos,
    Search,
}

pub struct AppState {
    pub messages: Vec<String>,
    pub authenticated: bool,
    pub active_menu_item: MenuItem,
    pub playlists: Vec<(String, String)>,
    pub search_result: Vec<(String, String, String, String)>,
    pub playlist_number_input: String,
    pub playlist_selection_mode: bool,
    pub search_input: String,
    pub search_attempted: bool,
    pub search_typing: bool,

    pub search_selection_mode: bool,
    pub search_number_input: String,
}

pub async fn event_handler(
    event: Event<KeyEvent>,
    state: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<bool> {
    match event {
        Event::Input(key_event) => match key_event.code {
            KeyCode::Char(c)
                if state.active_menu_item == MenuItem::Search && state.search_typing =>
            {
                state.search_input.push(c);
            }

            KeyCode::Char('s') => {
                state.active_menu_item = MenuItem::Search;
                state.search_input.clear(); // reset previous input
                state.search_result.clear(); // clear old results
                state.search_attempted = false;
                state.search_typing = true;
            }

            KeyCode::Enter if state.active_menu_item == MenuItem::Search && state.search_typing => {
                state.search_attempted = true;
                if let Some(token) = get_token() {
                    let maybe_results = utilities::search_videos(
                        token.access_token().secret(),
                        &state.search_input,
                    )
                    .await
                    .ok();

                    if let Some(new) = maybe_results {
                        state.search_result = new;
                    } else {
                        state
                            .messages
                            .push("❌ Failed to search videos".to_string());
                    }
                } else {
                    state.messages.push("❌ No token available".to_string());
                }
            }

            KeyCode::Char('b') if state.active_menu_item == MenuItem::Search => {
                state.search_selection_mode = true;
                state.search_number_input.clear();
            }

            KeyCode::Char(digit) if state.search_selection_mode && digit.is_ascii_digit() => {
                if state.search_number_input.len() < 2 {
                    state.search_number_input.push(digit);
                }
            }

            KeyCode::Enter if state.search_selection_mode => {
                if let Ok(idx) = state.search_number_input.parse::<usize>() {
                    if idx > 0 && idx <= state.search_result.len() {
                        if let Some(token) = get_token() {
                            if let Some((_title, _duration, _uploader, video_id)) =
                                state.search_result.get(idx - 1)
                            {
                                let access_token_str = token.access_token().secret();

                                utilities::play_song_by_id(video_id);

                                state.messages.push(format!("Playing video {}", video_id));
                            } else {
                                state.messages.push("Video not found.".to_string());
                            }
                        } else {
                            state.messages.push("No valid token.".to_string());
                        }
                    } else {
                        state
                            .messages
                            .push("Video number out of range.".to_string());
                    }
                } else {
                    state.messages.push("Invalid number input.".to_string());
                }

                state.search_selection_mode = false;
                state.search_number_input.clear();
            }
            KeyCode::Esc if state.search_selection_mode => {
                state.search_selection_mode = false;
                state.search_number_input.clear();
                state
                    .messages
                    .push("Search selection cancelled.".to_string());
            }
            KeyCode::Esc if state.active_menu_item == MenuItem::Search && state.search_typing => {
                state.search_typing = false;
                state.search_input.clear();
            }

            KeyCode::Backspace
                if state.active_menu_item == MenuItem::Search && state.search_typing =>
            {
                state.search_input.pop();
            }

            KeyCode::Char('q') => {
                disable_raw_mode()?;
                terminal.show_cursor()?;
                return Ok(true);
            }

            KeyCode::Char('a') => {
                state.messages.clear();
                state.authenticated = false;
                state.active_menu_item = MenuItem::Account;
            }

            KeyCode::Char('c') => state.active_menu_item = MenuItem::Commands,
            KeyCode::Char('h') => state.active_menu_item = MenuItem::Home,
            KeyCode::Char('v') => state.active_menu_item = MenuItem::Videos,

            KeyCode::Char('p') => {
                state.active_menu_item = MenuItem::Playlists;

                let maybe_playlists = if let Some(token) = get_token() {
                    utilities::list_playlists(token.access_token().secret())
                        .await
                        .ok()
                } else {
                    let _ = std::fs::write("tui_debug.log", "No token available\n");
                    None
                };

                if let Some(new) = maybe_playlists {
                    state.playlists = new;
                } else {
                    state
                        .messages
                        .push("❌ Failed to fetch playlists".to_string());
                }
            }

            KeyCode::Char('b') => {
                if state.active_menu_item == MenuItem::Playlists {
                    state.active_menu_item = MenuItem::Playlists;
                    state.playlist_selection_mode = true;
                    state.playlist_number_input.clear();
                }
            }

            KeyCode::Char(digit) if state.playlist_selection_mode && digit.is_ascii_digit() => {
                if state.playlist_number_input.len() < 2 {
                    state.playlist_number_input.push(digit);
                }
            }

            KeyCode::Enter if state.playlist_selection_mode => {
                if let Ok(idx) = state.playlist_number_input.parse::<usize>() {
                    if idx > 0 && idx <= state.playlists.len() {
                        if let Some(token) = get_token() {
                            if let Some(playlist) = state.playlists.get(idx - 1) {
                                let access_token_str = token.access_token().secret();
                                play_playlist(access_token_str, &playlist.1).await?;
                                state
                                    .messages
                                    .push(format!("Playing playlist: {}", playlist.0));
                            } else {
                                state.messages.push("Playlist not found.".to_string());
                            }
                        } else {
                            state.messages.push("No valid token.".to_string());
                        }
                    } else {
                        state
                            .messages
                            .push("Playlist number out of range.".to_string());
                    }
                } else {
                    state.messages.push("Invalid number input.".to_string());
                }
                state.playlist_selection_mode = false;
                state.playlist_number_input.clear();
            }

            KeyCode::Esc if state.playlist_selection_mode => {
                state.playlist_selection_mode = false;
                state.playlist_number_input.clear();
                state
                    .messages
                    .push("Playlist selection cancelled.".to_string());
            }

            KeyCode::Char('ű') => state.active_menu_item = MenuItem::Search,
            _ => {}
        },
        _ => {}
    }

    Ok(false)
}
