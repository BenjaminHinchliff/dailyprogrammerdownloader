const AUTHORIZE_URL: &str = "https://www.reddit.com/api/v1/authorize?client_id=no8ICo67XIf_dQ&response_type=token&state=hahalol&redirect_uri=http://localhost:3000/&scope=read";

fn main() {
    webbrowser::open(AUTHORIZE_URL).expect("failed to open url for reddit oauth");
}
