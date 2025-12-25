//app_state is the struct resposible for storing variables that control the app
//and handles keypress events that change those variables

use crate::colors::load_theme_from_file;
use crate::colors::{self, Theme};

use crate::utilities;
use crate::utilities::play_playlist;
use anyhow::Result;
use crossterm::event::KeyEvent;
use crossterm::{event::KeyCode, terminal::disable_raw_mode};
use oauth2::TokenResponse;
use std::fs;
use std::io;
use std::os::linux::raw::stat;
use std::path::Path;
use crate::authenticate::{load_token, OAuthToken};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
static OAUTH_TOKEN: OnceCell<Mutex<Option<OAuthToken>>> = OnceCell::new();



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
    Search,
}

pub struct AppState {
    pub messages: Vec<String>,
    pub authenticated: bool,
    pub active_menu_item: MenuItem,

    pub playlists: Vec<(String, String)>,
    pub playlist_number_input: String,
    pub playlist_selection_mode: bool,

    pub search_input: String,
    pub search_attempted: bool,
    pub search_typing: bool,
    pub search_result: Vec<(String, String, String, String)>,
    pub search_selection_mode: bool,
    pub search_number_input: String,

    pub selected_theme: Theme,
    pub themes: Vec<String>,
    pub theme_selection_mode: bool,
    pub theme_number_input: String,
    pub theme_selected_path: String,
}

//main keypress event handler for the tui
pub async fn event_handler(
    event: Event<KeyEvent>, //the event occuring
    state: &mut AppState, // the app state instance itself for changing variables
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, // the terminal instance

    ) -> Result<bool> //returns a Result (Ok or Err) for error handling

    //handling token errors 
    {
    if let Err(e) = load_and_set_token() {
        eprintln!("Failed to load token: {}", e);
    }

    //hanling keypress events
    match event {
        Event::Input(key_event) => match key_event.code {
            //saving search input into app state if search  is active and the user is typing
            KeyCode::Char(c)
                if state.active_menu_item == MenuItem::Search && state.search_typing =>
            {
                state.search_input.push(c);
            }
            
            //going into search mode if s is pressed, clearing previous search data too
            KeyCode::Char('s') => {
                state.active_menu_item = MenuItem::Search;
                state.search_input.clear(); // reset previous input
                state.search_result.clear(); // clear old results
                state.search_attempted = false;
                state.search_typing = true;
            }

            //starts the search itself if search mode is active and the user is typing

            KeyCode::Enter if state.active_menu_item == MenuItem::Search && state.search_typing => 
            {
            
                state.search_attempted = true;
                //retriving oauth token for the search
                if let Some(token)= get_token()                
                    {
                    let maybe_results = utilities::search_videos(token.access_token().secret(),
                        &state.search_input).await.ok();
                        //if the search_videos method returns something, it sets search_result to the results
                    if let Some(new) = maybe_results 
                        {
                        state.search_result = new;
                        }
                    //sends error message if the search fails    
                    else 
                        {
                        state.messages.push("Failed to search videos".to_string());
                        }
                    }

                else 
                    {
                    state.messages.push("No token available".to_string());
                    }
            }
            //selection for search items
            KeyCode::Char('b') if state.active_menu_item == MenuItem::Search => 
            {
                state.search_selection_mode = true;
                state.search_number_input.clear();
            }
            //saving the number of search results for playback
            KeyCode::Char(digit) if state.search_selection_mode && digit.is_ascii_digit() => 
            {
                if state.search_number_input.len() < 2 
                {
                    state.search_number_input.push(digit);
                }
            }
            
            //starting playback after selecting a search result by number
            KeyCode::Enter if state.search_selection_mode => 
            {
                if let Ok(idx) = state.search_number_input.parse::<usize>() 
                {
                    //only attempting playback if the index makes sense
                    if idx > 0 && idx <= state.search_result.len() 
                    {
                            //getting video and starting playback
                            if let Some((_title, _duration, _uploader, video_id)) =
                                state.search_result.get(idx - 1)
                            {
                                utilities::play_song_by_id(video_id);
                                state.messages.push(format!("Playing video {}", video_id));
                            } 
                            //error handling
                            else 
                            {
                                state.messages.push("Video not found.".to_string());
                            }
                    
                    }
                    //error handling here too 
                    else {
                        state.messages.push("Video number out of range.".to_string());
                    }
                } 
                else 
                {
                    state.messages.push("Invalid number input.".to_string());
                }
                //resetting selection variables
                state.search_selection_mode = false;
                state.search_number_input.clear();
            }

            //cancellation of selection by pressing Esc
            KeyCode::Esc if state.search_selection_mode => 
            {
                state.search_selection_mode = false;
                state.search_number_input.clear();
                state.messages.push("Search selection cancelled.".to_string());
            }
            //cancellation of searching
            KeyCode::Esc if state.active_menu_item == MenuItem::Search && state.search_typing => 
            {
                state.search_typing = false;
                state.search_input.clear();
            }
            //allows use of backspace while searching
            KeyCode::Backspace if state.active_menu_item == MenuItem::Search && state.search_typing =>
            {
                state.search_input.pop();
            }
            //quitting
            KeyCode::Char('q') => 
            {
                disable_raw_mode()?;
                terminal.show_cursor()?;
                return Ok(true);
            }
            //changing into accounts mode
            KeyCode::Char('a') => {
                state.messages.clear();
                state.authenticated = false;
                state.active_menu_item = MenuItem::Account;
            }

            //changing into commands mode
            KeyCode::Char('c') => state.active_menu_item = MenuItem::Commands,

            //changing into home mode
            KeyCode::Char('h') => 
            {
                state.active_menu_item = MenuItem::Home;
                let maybe_themes = utilities::get_theme_files();
                state.themes = maybe_themes?;
            }

            //changing to playlist mode and fetching users playlists
            KeyCode::Char('p') => {
                state.active_menu_item = MenuItem::Playlists;
                //attempts getting tokens and fetching playlists
                let maybe_playlists = if let Some(token) = get_token() 
                {
                    utilities::list_playlists(token.access_token().secret()).await.ok()
                } 
                else
                {
                    let _ = std::fs::write("tui_debug.log", "No token available\n");
                    None
                };
                //putting the result of list_playlist into state.playlist if it returns data
                if let Some(new) = maybe_playlists 
                {
                    state.playlists = new;
                }

                else 
                {
                    state.messages.push(" Failed to fetch playlists".to_string());
                }
            }
            //b is for binding in multiple modes
            KeyCode::Char('b') => 
            {
                //binding in playlist mode 
                if state.active_menu_item == MenuItem::Playlists 
                {
                    state.active_menu_item = MenuItem::Playlists;
                    state.playlist_selection_mode = true;
                    state.playlist_number_input.clear();
                }
                //and in home mode for themes to use
                if state.active_menu_item == MenuItem::Home 
                {
                    state.active_menu_item = MenuItem::Home;
                    state.theme_selection_mode = true;
                    state.playlist_number_input.clear();
                }
            }
            //saving numbers for playlist selection
            KeyCode::Char(digit) if state.playlist_selection_mode && digit.is_ascii_digit() => 
            {
                if state.playlist_number_input.len() < 2 
                {
                    state.playlist_number_input.push(digit);
                }
            }

            //saving numbers for theme selection
            KeyCode::Char(digit) if state.theme_selection_mode && digit.is_ascii_digit() => 
            {
                if state.theme_number_input.len() < 2 
                {
                    state.theme_number_input.push(digit);
                }
            }
            //changing theme by pressing enter after specifying number
            KeyCode::Enter if state.theme_selection_mode => 
            {
                //parsing input
                if let Ok(idx) = state.theme_number_input.parse::<usize>()
                {   //seeing if number is smaller or equal to the number of themes present
                    if idx > 0 && idx <= state.themes.len()
                    {   //getting path and the themes 
                        if let Some(path) = state.themes.get(idx - 1) 
                        {
                            match colors::load_theme_from_file(path) 
                            {   //if theme selection is successful, state is updated
                                Ok(new_theme) => 
                                {
                                    state.selected_theme = new_theme;
                                    state.messages.push(format!("Theme {} loaded successfully.", path));
                                }
                                Err(e) => 
                                {
                                    state.messages.push(format!("Failed to load theme: {}", e));
                                }
                            }
                        }
                        //error handling from here
                        else 
                        {
                            state.messages.push("Theme not found.".to_string());
                        }
                    } 
                    else 
                    {
                        state.messages.push("Theme number out of range.".to_string());
                    }
                } 
                else 
                {
                    state.messages.push("Invalid number input.".to_string());
                }
                state.theme_selection_mode = false;
                state.theme_number_input.clear();
            }

            //pretty much the same logic but for playlist selection 
            KeyCode::Enter if state.playlist_selection_mode => 
            {
                //if the chars entered are numbers and can be mapped to a playlist, playback will start
                if let Ok(idx) = state.playlist_number_input.parse::<usize>() 
                {
                    if idx > 0 && idx <= state.playlists.len() 
                    {
                        if let Some(token) = get_token() 
                        {
                            if let Some(playlist) = state.playlists.get(idx - 1) 
                            {
                                let access_token_str = token.access_token().secret();
                                play_playlist(access_token_str, &playlist.1).await?;
                                state.messages.push(format!("Playing playlist: {}", playlist.0));
                            }
                            //else statements from here are error handling 
                            else 
                            {
                                state.messages.push("Playlist not found.".to_string());
                            }
                        } 
                        else 
                        {
                            state.messages.push("No valid token.".to_string());
                        }
                    } 
                    else 
                    {
                        state.messages.push("Playlist number out of range.".to_string());
                    }
                } else {
                    state.messages.push("Invalid number input.".to_string());
                }
                state.playlist_selection_mode = false;
                state.playlist_number_input.clear();
            }
            //Esc to stop playlist selection
            KeyCode::Esc if state.playlist_selection_mode => 
            {
                state.playlist_selection_mode = false;
                state.playlist_number_input.clear();
                state.messages.push("Playlist selection cancelled.".to_string());
            },
            //do nothing on other key events
            _ => {}
        },
        //do nothing on any other events
        _ => {}
    }

    Ok(false)
}

//loads theme from file
pub fn load_and_set_theme_from_file(path: &str) -> Result<Theme> {
    let json = fs::read_to_string(path)?;
    let theme: Theme = serde_json::from_str(&json)?;
    Ok(theme)
}

pub fn set_token(token: OAuthToken) {
    OAUTH_TOKEN
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .replace(token);
}

pub fn get_token() -> Option<OAuthToken> {
    OAUTH_TOKEN.get()?.lock().unwrap().clone()
}

pub fn load_and_set_token() -> anyhow::Result<()> {
    let token = load_token()?;
    set_token(token);
    Ok(())
}
