use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use constants::LETTERS;
use game::{Game, GameId};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use serde::Deserialize;
use tinyrand::{RandRange, Seeded, StdRand};
use tokio::sync::RwLock;
use uuid::Uuid;
use word::WordState;

use crate::word::LetterState;

mod constants;
mod game;
mod storage;
mod word;

struct AppState {
    words: Vec<&'static str>,
    games: RwLock<HashMap<GameId, Game>>,
}

impl AppState {
    async fn new() -> Arc<AppState> {
        let words: Vec<&'static str> = include_str!("../words.txt").lines().collect();

        // Try and load the save data from disk
        match storage::load().await {
            Ok(save_data) => {
                return Arc::new(AppState {
                    words,
                    games: RwLock::new(
                        save_data
                            .games
                            .into_iter()
                            .map(|g| (g.id, g))
                            .collect::<HashMap<GameId, Game>>(),
                    ),
                });
            }
            Err(err) => println!("{err}"),
        }
        Arc::new(AppState {
            words,
            games: RwLock::new(HashMap::new()),
        })
    }
    async fn get_save_data(&self) -> Vec<Game> {
        self.games
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>()
    }
}

pub async fn run() {
    let state = AppState::new().await;

    let app = Router::new()
        .route("/", get(index))
        .route("/new_game", get(new_game))
        .route("/game/:id", get(game))
        .route("/games", get(games))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3242").await.unwrap();

    println!("🚀 Server Started: 0.0.0.0:3242 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn base(content: Markup) -> Markup {
    let style = r#"
body {
    font-family: 'Roboto Mono', monospace;
}
th {
    font-size: 12px;
}
td {
    font-size: 10px;
}
"#;

    let scripts = r#"
document.addEventListener('dblclick', function(event) {
    event.preventDefault();
}, { passive: false });
"#;

    html! {
        (DOCTYPE)
        head {
            html lang="en" data-framework="htmx";
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=3, maximum-scale=1, user-scalable=no" {}
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" {}
            script src="//unpkg.com/alpinejs" defer {}
            script src="https://unpkg.com/htmx.org@1.9.10" {}
            script { (scripts) }
            style { (style) }
        }
        body {
            div class="container p-3" {
                (content)
            }
        }
    }
}

async fn index() -> Markup {
    base(html! {
        div class="mx-auto" style="max-width:400px;" {
            div .card {
                div .card-body {
                    h1 .card-title { "📕 Wordle" }
                    p .card-text {
                        "Experience a thrilling game where you have six attempts to guess a secret five-letter word, using strategic guesses and clever deduction!"
                    }
                    small .card-text .text-secondary { "Made by " a href="https://iggyzuk.com/" { "Iggy Zuk" } }
                    div class="text-center" {
                        (new_game_btn_markup())
                        (all_games_btn_markup())
                    }
                }
            }
        }
        // Replace this with all games
        div #all-games { }
    })
}

async fn new_game(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Pick a random word.
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut rand = StdRand::seed(seed);
    let random_index = rand.next_range(0..state.words.len());

    let word = state.words[random_index].to_string();

    // Create a new game with a unique id.
    let id = Uuid::new_v4();

    // Drop write lock at end of block
    {
        let mut games = state.games.write().await;
        games.insert(id, Game::new(id, word.clone()));
    }

    // Save state with new game
    match storage::save(state.get_save_data().await).await {
        Ok(_) => {}
        Err(err) => panic!("{err}"),
    }

    println!("id: {id}, word: {word}");

    // Redirect to: /game/uuid
    Redirect::to(format!("/game/{}", id).as_str())
}

#[derive(Deserialize)]
struct GuessQuery {
    guess: Option<String>,
}

async fn game(
    State(state): State<Arc<AppState>>,
    Path(game_id): Path<GameId>,
    Query(query): Query<GuessQuery>,
) -> Markup {
    // todo: create alpine data object for this
    let js = PreEscaped(
        r#"
function addLetter(value, data) {
    const letter = value.toLowerCase();
    if(letter.length === 1 && data.letters.length < 5) {
        data.letters.push(letter);
    }
}
function removeLetter(data) {
    if(data.letters.length > 0) {
        data.letters.pop();
    }
}
"#,
    );

    let short_id = game_id.to_string().chars().take(8).collect::<String>();

    let markup = match state.games.write().await.get_mut(&game_id) {
        Some(game) => {
            let mut valid_word = true;

            if let Some(guess) = &query.guess {
                let guess = guess.trim().to_lowercase();
                if state.words.contains(&guess.as_str()) {
                    game.add_guess(guess);
                } else {
                    valid_word = false;
                }
            }

            base(html! {

                script { (js) }

                h1 { "📕 " a hx-boost="true" href="/" .text-dark { "Wordle" } }
                p { (short_id) }

                // Is this a guess attempt?
                @if let Some(guess) = query.guess {
                    // h3 { "Guess: " (guess) }

                    @if valid_word == false {
                        // Toast – maybe need to use hx-swap-oob="true"
                        div
                        role="alert"
                        #toast .alert .alert-danger .position-absolute .top-0 .start-0
                        x-data="{ show: true }" x-show="show"
                        x-init="setTimeout(() => show = false, 2000)" "x-transition.duration.500ms"
                        { (guess)" is not a valid word" }
                    }
                }

                div
                .m-3
                x-data="{
                    letters: [], 
                    combine: function() { return this.letters.join(''); }, 
                    fill: function() { return this.letters.concat(Array(5 - this.letters.length).fill('-')).join(''); } 
                }"
                "@keydown.window"="addLetter($event.key, $data)" // add letter on `key`
                "@keydown.backspace.window"="removeLetter($data)" // remove letter on `backspace`
                "@click-letter.window"="addLetter($event.detail.letter, $data)" // add letter on `click`
                "@click-erase.window"="removeLetter($data)" // remove letter on `click`
                {
                    @let mut show_dynamic = !game.is_complete();
                    @for guess in &game.get_guesses() {
                        @if let Some(guess) = guess {
                            (WordState::guess(guess, &game.word))
                        } @else if show_dynamic {
                            @let _ = show_dynamic = false;
                            (dynamic_word_markup())
                        } @else {
                            (WordState::empty())
                        }
                    }

                    // The form for guessing.
                    // Note: we replace the entire body, could look into hx-select and target a specific id.
                    @if !game.is_complete() {
                        form
                        #guess-form .text-center
                        hx-get={"/game/"(game_id)}
                        hx-target="body"
                        hx-trigger="keydown[keyCode==13] from:body, click-guess from:body" // submit on `enter` or `click`
                        { input type="hidden" x-model="combine" id="guess" name="guess" {} }
                    } @else {
                        div class="text-center" {
                            h3 { "the word was: " b { (game.word) } }
                            (new_game_btn_markup())
                        }
                    }
                }

                (available_letters_markup(&game.get_available_letters()))
            })
        }
        None => base(html! {
            div class="text-center p-2" {
                h1 { "Game doesn't exist!" }
                p { (game_id) }
                (new_game_btn_markup())
            }
        }),
    };

    // Save the state after the write lock is dropped
    match storage::save(state.get_save_data().await).await {
        Ok(_) => {}
        Err(err) => panic!("{err}"),
    }

    markup
}

async fn games(State(state): State<Arc<AppState>>) -> Markup {
    let games = state.games.read().await;

    let mut games_sorted = games.values().collect::<Vec<_>>();
    games_sorted.sort_by_key(|g| g.created);
    games_sorted.reverse();

    html! {
        table class="table" {
            thead {
                tr {
                    th scope="col" { "Word" }
                    th scope="col" { "Final" }
                    th scope="col" { "Guess" }
                    th scope="col" { "Date" }
                    th scope="col" { "Link" }
                }
            }
            tbody {
                @for game in games_sorted {
                    @let complete = game.is_complete();
                    @let victory = game.is_victory();
                    @let loss = game.is_loss();
                    @let guesses = game.guesses.len();
                    @let short_id = game.id.to_string().chars().take(8).collect::<String>();
                    @let last_guess = game.guesses.iter().last();
                    tr .table-warning[victory] .table-danger[loss] .fw-bold[complete] {
                        @if complete {
                            td .text-warning[victory] .text-danger[loss] { (game.word) }
                        } @else {
                            td { "?????" }
                        }
                        @if last_guess.is_some() {
                            td .text-warning[victory] .text-danger[loss] { (last_guess.unwrap()) }
                        } @else {
                            td { "-----" }
                        }
                        td { (guesses)"/6" }
                        td { (game.created.unwrap().format("%y/%m/%d")) }
                        td { a hx-boost="true" href={"/game/"(game.id)} { (short_id) } }
                    }
                }
            }
        }
    }
}

fn new_game_btn_markup() -> Markup {
    html! { button hx-target="body" hx-push-url="true" hx-get="/new_game" class="btn btn-primary m-2" { "⭐️ Play" } }
}

fn all_games_btn_markup() -> Markup {
    html! { button hx-target="#all-games" hx-get="/games" class="btn btn-warning m-2" { "📘 Games" } }
}

fn available_letters_markup(available: &Vec<char>) -> Markup {
    let mut chars = LETTERS.chars().collect::<Vec<_>>();
    let mut segments = vec![];

    segments.push(chars.drain(..10).map(|c| (c, None)).collect::<Vec<_>>());
    segments.push(chars.drain(..9).map(|c| (c, None)).collect::<Vec<_>>());

    // Last row adds two special buttons
    let mut last_row = chars.drain(..).map(|c| (c, None)).collect::<Vec<_>>();
    last_row.push(('✅', Some("click-guess")));
    last_row.insert(0, ('❌', Some("click-erase")));
    segments.push(last_row);

    html! {
        div x-data {
            @for segment in segments {
                div class="d-flex flex-wrap gap-1 mb-1 justify-content-center" {
                    @for letter in segment {
                        @if let Some(event) = letter.1 {
                            // Special buttons: guess/erase – they dispatch events
                            div
                            .p-2 .btn .btn-outline-primary
                            "@mousedown"={"$dispatch('"(event)"')"}
                            { (letter.0) }
                        } @else {
                            // Basic buttons – they dispatch events
                            @let class = if available.contains(&letter.0) {
                                "p-2 btn btn-primary"
                            } else {
                                "p-2 btn btn-secondary"
                            };
                            div
                            class=(class)
                            "@mousedown"={"$dispatch('click-letter', { letter: '"(letter.0)"' })"}
                            { (letter.0) }
                        }
                    }
                }
            }
        }
    }
}

impl Render for WordState {
    fn render(&self) -> Markup {
        html! {
            div class = "d-flex justify-content-center gap-1 mb-1" {
                @for letter in &self.letters {
                    @match letter.state {
                        LetterState::Correct => {
                            div class="p-2 bg-success text-white border" { (letter.id) }
                        },
                        LetterState::WrongPlace => {
                            div class="p-2 bg-warning text-white border" { (letter.id) }
                        },
                        LetterState::Wrong => {
                            div class="p-2 bg-secondary text-white border" { (letter.id) }
                        },
                        LetterState::Empty => {
                            div class="p-2 bg-light text-dark border" { (letter.id) }
                        }
                    }
                }
            }
        }
    }
}

fn dynamic_word_markup() -> Markup {
    html! {
        div class = "d-flex justify-content-center gap-1 mb-1" {
            template x-for="letter in fill" {
                div
                .p-2 .bg-light .text-dark .border
                x-bind:class="{ 'border-primary': letter == '-' }"
                x-text="letter"
                { }
            }
        }
    }
}
