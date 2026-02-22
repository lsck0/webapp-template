use maud::{DOCTYPE, Markup, html};

pub async fn index_html() -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                script src="https://cdn.tailwindcss.com" {}
                script defer src="https://unpkg.com/htmx.org" {}
                script defer src="https://unpkg.com/alpinejs" {}
                title { "Webapp Template" }
            }
            body {
                p { "Hello, World!" }
            }
        }
    }
}
