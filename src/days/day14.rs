use axum::{
    extract::Json,
    response::Html,
    routing::post,
    Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct TemplateContent {
    content: String,
}

fn render_header_from_template(content: &str) -> String {
    format!(
r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>"#,
        content
    )
}

async fn render_unsafe(Json(content): Json<TemplateContent>) -> Html<String> {
    let html = render_header_from_template(&content.content);
    Html(html)
}

async fn render_safe(Json(content): Json<TemplateContent>) -> Html<String> {
    let mut cleaned = String::new();
    for c in content.content.chars() {
        match c {
            '<' => { cleaned.push_str("&lt;"); },
            '>' => { cleaned.push_str("&gt;"); },
            '&' => { cleaned.push_str("&amp;"); },
            '"' => { cleaned.push_str("&quot;"); },
            '\'' => { cleaned.push_str("&apos;"); },
            _ => { cleaned.push(c); } 
        };
    }
    let html = render_header_from_template(&cleaned);
    Html(html)
}

pub fn html_reindeer_route() -> Router {
    Router::new().route("/unsafe", post(render_unsafe))
        .route("/safe", post(render_safe))
}