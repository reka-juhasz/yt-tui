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
    pub playlist_number_input: String,
    pub playlist_selection_mode: bool,
}

pub async fn event_handler(
    event: Event<KeyEvent>,
    state: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<bool> {
    match event {
        Event::Input(key_event) => match key_event.code {
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
                let _ = std::fs::write("tui_debug.log", "Pressed 'p' key handler triggered\n");
                state.active_menu_item = MenuItem::Playlists;

                let maybe_playlists = if let Some(token) = get_token() {
                    let _ =
                        std::fs::write("tui_debug.log", "Token found, calling list_playlists\n");
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
                state.active_menu_item = MenuItem::Playlists;
                state.playlist_selection_mode = true;
                state.playlist_number_input.clear();
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

            KeyCode::Char('s') => state.active_menu_item = MenuItem::Search,
            _ => {}
        },
        _ => {}
    }

    Ok(false)
}
