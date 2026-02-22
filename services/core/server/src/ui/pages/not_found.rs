use axum::http::Uri;
use maud::{DOCTYPE, Markup, html};

pub async fn not_found_html(uri: Uri) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                script src="https://cdn.tailwindcss.com" {}
                title { "Not Found" }
            }
            body {
                p { (uri) " Not Found" }
            }
        }
    }
}
