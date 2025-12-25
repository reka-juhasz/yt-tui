use anyhow::Result;
use serde::Deserialize;
use std::fs;
use tui::style::Color;

//newtype wrapper around tui::style::Color with custom Deserialize impl
#[derive(Debug, Clone, Copy)]
pub struct MyColor(pub Color);

impl<'de> Deserialize<'de> for MyColor 
    {
    fn deserialize<D>(deserializer: D) -> Result<MyColor, D::Error> where D: serde::Deserializer<'de>,
        {
        let s = String::deserialize(deserializer)?;
        parse_color(&s).map(MyColor).ok_or_else(|| serde::de::Error::custom(format!("Invalid color: {}", s)))
        }
    }

//theme has colors for most ui components, allowing for customization
#[derive(Debug, Deserialize)]
pub struct Theme 
{
    pub tui_lines: MyColor,
    pub active_menu_item: MyColor,
    pub other_menu_items: MyColor,
    pub tabs_basic: MyColor,
    pub tabs_highlight: MyColor,

    //////////////////////
    pub home_text: MyColor,
    pub home_box: MyColor,
    pub playlist_number: MyColor,
    pub playlist_name: MyColor,
    pub playlist_box: MyColor,
    pub account_info: MyColor,
    pub account_link: MyColor,
    pub account_auth_success: MyColor,
    pub account_auth_failure: MyColor,
    pub account_box: MyColor,
    pub command_text_odd: MyColor,
    pub command_text_even: MyColor,
    pub command_box: MyColor,
    pub search_box: MyColor,
    pub search_number: MyColor,
    pub search_name: MyColor,
    pub search_uploader: MyColor,
    pub search_duration: MyColor,
}
//theme impl for storing the themes read in from .json file
impl Theme 
{
    pub fn new() -> Self 
    {
        Self 
        {
            tui_lines: MyColor(Color::White),
            active_menu_item: MyColor(Color::White),
            other_menu_items: MyColor(Color::White),
            tabs_basic: MyColor(Color::White),
            tabs_highlight: MyColor(Color::White),
            home_text: MyColor(Color::White),
            home_box: MyColor(Color::White),
            playlist_number: MyColor(Color::White),
            playlist_name: MyColor(Color::White),
            playlist_box: MyColor(Color::White),
            account_info: MyColor(Color::White),
            account_link: MyColor(Color::White),
            account_auth_success: MyColor(Color::Green),
            account_auth_failure: MyColor(Color::Red),
            account_box: MyColor(Color::White),
            command_text_odd: MyColor(Color::Gray),
            command_text_even: MyColor(Color::White),
            command_box: MyColor(Color::White),
            search_box: MyColor(Color::White),
            search_number: MyColor(Color::White),
            search_name: MyColor(Color::White),
            search_uploader: MyColor(Color::White),
            search_duration: MyColor(Color::White),
        }
    }
}

/// parsing custom colors and also can process rgb values 
fn parse_color(s: &str) -> Option<Color> 
{
    let s = s.trim().to_lowercase();
    match s.as_str() 
    {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::Gray),
        "darkgray" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => {
            if let Some(rgb) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(")")) 
            {
                let parts: Vec<_> = rgb.split(',').collect();
                if parts.len() == 3 
                {
                    let r = parts[0].trim().parse().ok()?;
                    let g = parts[1].trim().parse().ok()?;
                    let b = parts[2].trim().parse().ok()?;
                    return Some(Color::Rgb(r, g, b));
                }
            }
            None
        }
    }
}

/// Loads a Theme from a JSON file
pub fn load_theme_from_file(path: &str) -> Result<Theme> 
{
    let json = fs::read_to_string(path)?;
    let theme: Theme = serde_json::from_str(&json)?;
    Ok(theme)
}