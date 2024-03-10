use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
};
use maud::{html, Markup, DOCTYPE};
use tokio::sync::RwLock;

struct AppState {
    articles: RwLock<Vec<Article>>,
}

impl AppState {
    fn new() -> Arc<Self> {
        let mut articles = vec![];
        for i in 0..95 {
            articles.push(Article {
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                photo_url: format!("https://picsum.photos/seed/{}/300", i + 1),
            })
        }

        Arc::new(AppState {
            articles: RwLock::new(articles),
        })
    }
}

struct Article {
    title: String,
    content: String,
    photo_url: String,
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(index))
        .route("/articles/:page", get(articles))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5443").await.unwrap();

    println!("🚀 Server Started: 0.0.0.0:5443 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn base(content: Markup) -> Markup {
    let style = include_str!("../style.css");

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Infinite Scroll (htmx)" }
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                style { (style) }
            }
            body { (content) }
        }
    }
}

async fn index(State(state): State<Arc<AppState>>) -> Markup {
    let articles = state.articles.read().await;
    base(articles_fragment(&articles, 0))
}

async fn articles(State(state): State<Arc<AppState>>, Path(page): Path<usize>) -> Markup {
    let articles = state.articles.read().await;
    articles_fragment(&articles, page)
}

/// HTML fragment of articles
fn articles_fragment(articles: &Vec<Article>, page: usize) -> Markup {
    const PAGE_SIZE: usize = 10;

    html! {
        div id="articles" {
            @for i in 0..PAGE_SIZE {
                div class="article" {
                @if let Some(article) = articles.iter().nth(page * PAGE_SIZE + i) {
                        // on reveal of the last article we will add more articles to the #articles div
                        @if i == PAGE_SIZE - 1{
                            span hx-get=(format!("/articles/{}", page + 1)) hx-swap="beforeend" hx-target="#articles" hx-select=".article" hx-trigger="revealed" {}
                        }

                        // title + image + content
                        b { nobr { (article.title) } }
                        a class="image-container" href=(article.photo_url) {
                            img width="300" height="300" src=(article.photo_url) alt="..." {}
                        }
                        i { (article.content) }
                    } @else {
                        // no more articles left ... (spawn a div anyway as an example)
                        i { "..." }
                    }
                }
            }
        }
    }
}
