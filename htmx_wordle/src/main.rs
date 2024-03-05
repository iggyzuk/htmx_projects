use std::{sync::Arc, time::SystemTime};

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use tinyrand::{RandRange, Seeded, StdRand};
use tokio::sync::RwLock;
use uuid::Uuid;

mod word;

struct AppState {
    games: RwLock<Vec<Game>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        games: RwLock::new(vec![]),
    });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(play))
        .route("/guess", post(guess))
        .with_state(state);

    // run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3242").await.unwrap();

    println!("🚀 Server Started: 0.0.0.0:3242 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        head {
            html lang="en" data-framework="htmx";
            meta charset="utf-8";
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" {}
            script src="//unpkg.com/alpinejs" defer {}
            script src="https://unpkg.com/htmx.org@1.9.10" {}
        }
        body {
            div class="container" {
                (content)
            }
        }
    }
}

async fn play() -> Markup {
    let words = include_str!("../words.txt");
    let lines = words.lines().collect::<Vec<_>>();

    let id = Uuid::new_v4();

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut rand = StdRand::seed(seed);
    let random_index = rand.next_range(0..lines.len());

    let word = lines[random_index];

    let html = html! {
        h1 { "Wordle: " (word) }
        p { (id) }

        div
        id="toast"
        class="alert alert-danger"
        role="alert"
        // hx-swap-oob="true"
        x-data="{ show: true }"
        x-show="show"
        x-init="setTimeout(() => show = false, 3000)"
        "x-transition.duration.500ms"
        {
            "X is not a valid word"
        }

        button { "Play" }

        form {
            input hidden="true" name="word" value="CRAZY" {}
            button hx-post="/check_word" { "Check Word" }
        }

        form {
            input hidden="true" name="word" value="PIANO" {}
            button hx-post="/guess" { "Guess" }
        }
    };

    page(html)
}

#[derive(Deserialize)]
struct Guess {
    word: String,
}

async fn guess(State(state): State<Arc<AppState>>, Form(data): Form<Guess>) -> Markup {
    let mut games = state.games.write().await;

    games.push(Game {
        id: Uuid::new_v4(),
        word: "Nothing".to_string(),
        guesses: vec![],
        completed: false,
    });

    // Check that the word is even valid
    html! {
        div { "Guess: " (data.word) " -> Games: "(games.len()) }
    }
}

#[derive(Deserialize)]
struct CheckWord {
    word: String,
}

async fn check_word(Form(data): Form<CheckWord>) -> Markup {
    html! {
        div { "Real Word: " (data.word) }
    }
}

struct Game {
    id: Uuid,
    word: String,
    guesses: Vec<String>,
    completed: bool,
}

impl Game {
    fn add_guess(&mut self, word: String) {
        self.guesses.push(word);
    }

    fn get_available_letters(&self) -> Vec<char> {
        let mut letters = "abcdefghijklmnopqrstuvwxyz"
            .split("")
            .map(|s| s.parse::<char>().unwrap())
            .collect::<Vec<_>>();

        let used_chars = self.guesses.iter().flat_map(|x| x.chars());

        for used_char in used_chars {
            if letters.contains(&used_char) {
                letters.retain(|&c| c != used_char);
            }
        }

        letters

        // <div class="flex flex-col gap-2 items-center">
        // {LINES.map((line) => (
        //   <div class="flex gap-2">
        //     {line.map((letter) => (
        //       <div
        //         class={`flex items-center justify-center w-8 h-12 rounded-md ${
        //           available.includes(letter) ? "bg-gray-200" : "bg-gray-700 text-white"
        //         }`}
        //       >
        //         {letter.toUpperCase()}
        //       </div>
        //     ))}
        //   </div>
        // ))}
        // </div>
    }
}
