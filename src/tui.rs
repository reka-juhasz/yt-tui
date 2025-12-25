//the tui is responsible for initializing backend components and rendering crossterm widgets
//functionally, the tui_render is the main
use crate::app_state;
use crate::app_state::AppState;
use crate::app_state::Event;
use crate::app_state::MenuItem;
use crate::authenticate::authenticate;
use crate::colors::Theme;
use crate::render;
use anyhow::Result;
use crossterm::{event::{self, Event as CEvent},terminal::enable_raw_mode,};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tui::{backend::CrosstermBackend,layout::{Constraint, Direction, Layout},style::{Modifier, Style},text::{Span, Spans},widgets::{Block, Borders, Tabs},Terminal,};

impl From<MenuItem> for usize {
    //numbering the menu items with a usize (variable used for indexing collections)
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Account => 0,
            MenuItem::Commands => 1,
            MenuItem::Home => 2,
            MenuItem::Playlists => 3,
            MenuItem::Search => 4,
        }
    }
}
//main function
pub async fn tui_render() -> Result<()> {
    //creating a mutable (!!) appstate instance
    let mut state = AppState {
        //inital app state
        messages: vec![],
        authenticated: false,
        active_menu_item: MenuItem::Home,
        playlists: vec![],
        search_result: vec![],
        search_attempted: false,
        playlist_number_input: String::new(),
        playlist_selection_mode: false,
        search_input: String::new(),
        search_typing: false,
        search_number_input: String::new(),
        themes: vec![],
        search_selection_mode: false,
        theme_selection_mode: false,
        selected_theme: Theme::new(),
        theme_number_input: String::new(),
        theme_selected_path: "themes/blue_theme.json".to_string(),
    };

    state.selected_theme = app_state::load_and_set_theme_from_file(&state.theme_selected_path)?; 
    enable_raw_mode().expect("can run in raw mode"); //putting terminal in raw mode
    //sender and reciever events initialized,these allow communication between the user and the app
    let (tx, rx) = mpsc::channel(); 
    let tick_rate = Duration::from_millis(200);
    //cloning tx to avoid ownership issues
    let tx_input = tx.clone();

    let stdout = io::stdout();
    //backend for drawing onto the terminal
    let backend = CrosstermBackend::new(stdout); 
    let mut terminal = Terminal::new(backend)?; 
    terminal.clear()?; 
    //collection of menuitems
    let menu_titles = vec!["Account", "Commands", "Home", "Playlists", "Search"]; 

    thread::spawn(move || {
        //thread for calculating tics, moves tx to keep it in scope
        let mut last_tick = Instant::now();
        loop {
            //this is shamelessly stolen from a tui tutorial, sorry
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx_input.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                let _ = tx_input.send(Event::Tick);
                last_tick = Instant::now();
            }
        }
    });
    //new runtime
    let rt = Runtime::new()?;
    //assumes auth token is expired on startup
    let mut authenticated = false; 

    loop {
        if state.active_menu_item == MenuItem::Account && !authenticated {
            authenticated = true;
            state.messages.clear();

            let tx_msg = tx.clone();
            //handling the terminal while authenticating, diasbling raw mode to allow pasting of auth url
            rt.spawn(async move {
                //new thread spawn for authentication
                let _ = crossterm::terminal::disable_raw_mode();
                let result = authenticate(|msg| {let _ = tx_msg.send(Event::Message(msg.to_string()));}).await;
                //raw mode enabled ifauthentication successul
                let _ = crossterm::terminal::enable_raw_mode(); 

                if let Err(e) = result {
                    let _ = tx_msg.send(Event::Message(format!("Authentication error: {}", e)));
                }
            });
        }
        //drawing terminal begins here
        terminal.draw(|rect| 
            {
            let size = rect.size();
            //the layout is vertical with 2 parts, one that has a set height of 3 rows and the other takes the lest
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);
            //adding menu items, with colors and modifiers    
            let menu = menu_titles
                .iter()
                .map(|t| 
                    {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(state.selected_theme.active_menu_item.0)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(
                            rest,
                            Style::default().fg(state.selected_theme.other_menu_items.0),
                        ),
                    ])
                }).collect();
            //adding tabs with styling as well    
            let tabs = Tabs::new(menu)
                .select(state.active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(state.selected_theme.tabs_basic.0))
                .highlight_style(Style::default().fg(state.selected_theme.tabs_highlight.0))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            //match case for the currently active menu item    
            match state.active_menu_item 
            {
                MenuItem::Home => 
                {
                    rect.render_widget(
                        render::render_home(
                            &state.selected_theme,
                            &state.themes,
                            state.theme_selection_mode,
                            &state.theme_number_input,
                        ),
                        chunks[1],
                    );
                }
                MenuItem::Playlists => 
                {
                    rect.render_widget(
                        render::render_playlists(
                            &state.selected_theme,
                            &state.playlists,
                            state.playlist_selection_mode,
                            &state.playlist_number_input,
                        ),
                        chunks[1],
                    );
                }

                MenuItem::Account => 
                {
                    rect.render_widget(
                        render::render_accounts(&state.selected_theme, &state.messages),
                        chunks[1],
                    );
                }
                MenuItem::Search => 
                {
                    rect.render_widget(
                        render::render_search_prompt(&state.search_input),
                        chunks[1],
                    );
                    rect.render_widget(
                        render::render_search(
                            &state.selected_theme,
                            &state.search_result,
                            state.search_attempted,
                            state.search_selection_mode,
                            &state.search_number_input,
                        ),
                        chunks[1],
                    );
                }
                MenuItem::Commands => 
                {
                    rect.render_widget(render::render_commands(&state.selected_theme), chunks[1]);
                }
            }
        })?;
        //handling events
        match rx.recv()? 
        {   //input event handler from the app_state
            Event::Input(event) => 
            {
                if app_state::event_handler(Event::Input(event), &mut state, &mut terminal).await? 
                {
                    break Ok(());
                }
            }
            Event::Tick => {}
            Event::Message(msg) => state.messages.push(msg),
        }
    }
}