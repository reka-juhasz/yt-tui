This is a project written in Rust, that lets you use YouTube from the terminal!

#### Installation

In order to use the TUI you need to have Rust, mvp and yt-dlp installed and need to clone the repository to your system. 
Mvp and yt-dlp are usually installed by default, but might need to be installed manually on more minimal distros. 

#### Getting YouTube credentials 

After cloning this repository, you will need to configure a few things in order to be able to use the TUI, but it shouldn't take more than five minutes to do so.
This project uses the YouTube Data API, so you will need to follow these steps to get the necessary credentials to access the API and be able to use the TUI.

1. Go to the Google Cloud Console (https://console.cloud.google.com/) and create a new project.
2. On the left sidebar, enable the YouTube Data API v3 (APIs & Services > Library > YouTube Data API v3 > Enable)
3. Create OAuth 2.0 credentials (APIs and Services > Credentials > CREATE CREDENTIALS > OAuth client ID ). Make sure you generate OAuth 2.0 credentials and not an API key.
4. Fill out any additional requested information (the app should be external and the developer email should be your own).
5. Set the application type to "Desktop App"
6. Download the credentials and put them into the /src folder under the name credentials.txt
7. Add the Gmail address you want to use in the TUI as a test user (Audience > Test users)

#### Usage

After creating your credentials.txt, you can start the tui by running 'cargo run' in the folder you pasted your credentials. By pressing 'c' you will be taken to the commands menu, where you can see how you can navigate the TUI. After this, you should go to the Account page. This will open a tab in your browser, asking you to log into the YouTube. After authenticating successfully, you will be taken back to your homepage, and your top search bar will contain a long URL. From this URL, paste the code between "&code=" and "&scope" into the TUI and press enter. You may need to restart the TUI for changes to take effect, but after that, you should be able to use the TUI. The access tokens usually refresh themselves, but sometimes you might get prompted to authenticate again, though that is quite rare. 

#### Current capabilities 

This project is still actively in development, and as such, does not have all its functionalities implemented. Right now, you are able to log into your YouTube account, see your playlists, play them and skip songs in them. You are also able to search and play videos as well. In the future, I would like to add functionalities that make using the TUI feel nicer. Some of these planned features include:
  - Stopping and resuming playback 
  - Adjusting volume
  - Skipping videos and playing the previous video 
  - A nicer looking UI
Sadly, some features cannot be implemented because of the YouTube API doesn't support them, a good example for this is fetching the data from your Home feed or the Subscriptions feed. I am a university student, and while i will try my best to make this project as functional as I can, development might slow down after September.

#### Customization

By going to the Home tab and pressing 'h' again, you can see all the existing themes for the TUI. Press 'b' and enter the number of the theme you wish to use and press Enter. The theme change should be visible immediately. You can also define your own themes by putting them in the /src/themes folder. If you do so, please follow this .json format, to insure your theme is applicable and won't crash the TUI due to parsing errors:

{
  "tui_lines": "Rgb(0,120,200)",
  "active_menu_item": "Rgb(180,225,255)",
  "other_menu_items": "Rgb(0,105,185)",
  "tabs_basic": "Rgb(70,180,230)",
  "tabs_highlight": "Rgb(70,180,230)",
  "home_text": "Rgb(180,225,255)",
  "home_box": "Rgb(40,60,90)",
  "playlist_number": "Rgb(180,225,255)",
  "playlist_name": "Rgb(80,200,240)",
  "playlist_box": "Rgb(40,60,90)",
  "account_info": "Rgb(180,225,255)",
  "account_link": "Rgb(55,80,110)",
  "account_auth_success": "Rgb(210,245,255)",
  "account_auth_failure": "Rgb(100,130,160)",
  "account_box": "Rgb(40,60,90)",
  "command_text_odd": "Rgb(0,115,190)",
  "command_text_even": "Rgb(0,100,170)",
  "command_box": "Rgb(40,60,90)",
  "search_box": "Rgb(40,60,90)",
  "search_number": "Rgb(180,225,255)",
  "search_name": "Rgb(70,180,230)",
  "search_uploader": "Rgb(0,105,185)",
  "search_duration": "Rgb(220,250,255)"
}
