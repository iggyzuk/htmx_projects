use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use game::{short_id, Game, GameId, Letter};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use serde::Deserialize;
use tinyrand::{RandRange, Seeded, StdRand};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::game::{LetterState, WordState};

mod game;
mod storage;

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

    let address = "0.0.0.0:3242";

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn base(content: Markup) -> Markup {
    let style = include_str!("../style.css");
    let scripts = PreEscaped(include_str!("../scripts.js"));

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=3, maximum-scale=1, user-scalable=no" {}
                title { "Wordle (htmx)" }

                // Styles
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" {}
                style { (style) }

                // Htmx + Alpine
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="//unpkg.com/alpinejs" defer {}

                // Custom scripts
                script { (scripts) }

            }
            body {
                div class="container p-3" {
                    (content)
                }
            }
        }
    }
}

async fn index(State(state): State<Arc<AppState>>) -> Markup {
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
                        (all_games_btn_markup(state.games.read().await.len()))
                    }
                }
            }
        }
        // Replace this with all games
        div #all-games { }
    })
}

async fn new_game(State(state): State<Arc<AppState>>) -> Response {
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

    let game = Game::new(id, word.clone());

    // Drop write lock at end of block
    {
        let mut games = state.games.write().await;
        games.insert(id, game);
    }

    // Save state with new game
    match storage::save(state.get_save_data().await).await {
        Ok(_) => {}
        Err(err) => panic!("{err}"),
    }

    println!("id: {id}, word: {word}");

    // HX-Location
    // This response header can be used to trigger a client side redirection without
    // reloading the whole page. Instead of changing the page’s location it will act
    // like following a hx-boost link, creating a new history entry, issuing an ajax
    // request to the value of the header and pushing the path into history.
    Response::builder()
        .status(StatusCode::CREATED)
        .header("HX-Location", format!("/game/{id}"))
        .body(Body::empty())
        .unwrap()
}

#[derive(Deserialize)]
struct GuessQuery {
    guess: Option<String>,
}

async fn game(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(game_id): Path<GameId>,
    Query(query): Query<GuessQuery>,
) -> Markup {
    // Check headers if we should send a fragment or a full page back.
    let is_fragment = headers.get("hx-request").is_some()
        && headers
            .get("hx-target")
            .map_or(false, |target| target == "wordle-content");

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

            // Send the fragment or the full page.
            let fragment = game_fragment(&game, query.guess, valid_word);
            if is_fragment {
                fragment
            } else {
                base(html! {
                    h1 { "📕 " a hx-boost="true" href="/" .text-dark { "Wordle" } }
                    p { (short_id(game.id)) }
                    div #wordle-content {
                        (fragment)
                    }
                })
            }
        }
        None => base(html! {
            div class="text-center p-2" {
                h1 { "Game doesn't exist!" }
                p { (game_id) }
                (new_game_btn_markup())
            }
        }),
    };

    // Save the state after each guess
    match storage::save(state.get_save_data().await).await {
        Ok(_) => {}
        Err(err) => panic!("{err}"),
    }

    markup
}

fn game_fragment(game: &Game, guess: Option<String>, valid_word: bool) -> Markup {
    html! {

        // Is this a guess attempt?
        @if let Some(guess) = guess {
            // h3 { "Guess: " (guess) }

            @if valid_word == false {
                // Toast – maybe need to use hx-swap-oob="true"
                div
                role="alert"
                #toast .alert .alert-danger
                style="position: absolute; top: 10px; left: 10px;"
                x-data="{ show: true }" x-show="show"
                x-init="setTimeout(() => show = false, 2000)" "x-transition.duration.500ms"
                { b { (guess) }" is not a valid word" }
            }
        }

        div
        .m-3
        x-data="wordleDataObject"
        "@keydown.window"="addLetter($event.key)" // add letter on `key`
        "@keydown.backspace.window"="removeLetter()" // remove letter on `backspace`
        "@click-letter.window"="addLetter($event.detail.letter)" // add letter on `click`
        "@click-erase.window"="removeLetter()" // remove letter on `click`
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
                hx-get={"/game/"(game.id)}
                hx-target="#wordle-content"
                hx-swap="innerHTML"
                hx-trigger="keydown[keyCode==13] from:body, click-guess from:body" // submit on `enter` or `click`
                {
                    input
                    type="hidden"
                    x-model="combine"
                    id="guess"
                    name="guess"
                    {}
                }
            } @else {
                div class="text-center" {
                    h3 { "the word was: " b { (game.word) } }
                    (new_game_btn_markup())
                }
            }
        }

        (available_letters_markup(&game.get_available_letters()))
    }
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
                        td { a hx-boost="true" href={"/game/"(game.id)} { (short_id(game.id)) } }
                    }
                }
            }
        }
    }
}

fn new_game_btn_markup() -> Markup {
    html! { button hx-get="/new_game" hx-target="body" class="btn btn-primary m-2" { "⭐️ Play" } }
}

fn all_games_btn_markup(count: usize) -> Markup {
    html! { button hx-target="#all-games" hx-get="/games" class="btn btn-warning m-2" { "📘 Games " small { (count) } } }
}

fn available_letters_markup(available_letters: &HashMap<char, Letter>) -> Markup {
    let mut chars = game::LETTERS.chars().collect::<Vec<_>>();
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
                            @let class = match available_letters[&letter.0].state {
                                LetterState::Correct => "btn btn-success",
                                LetterState::WrongPlace => "btn btn-warning",
                                LetterState::Wrong => "btn btn-secondary",
                                LetterState::Empty => "btn btn-primary",
                            };

                            div
                            class={"p-2 " (class)}
                            "@mousedown"={"$dispatch('click-letter', { letter: '"(letter.0)"' })"}
                            { (letter.0) }
                        }
                    }
                }
            }
        }
    }
}

/// The markup for a word-state, it shows which letters are correct, are in wrong place, or simply wrong.
impl Render for WordState {
    fn render(&self) -> Markup {
        html! {
            div class = "d-flex justify-content-center gap-1 mb-1" {
                @for letter in &self.letters {
                    @let class = match letter.state {
                        LetterState::Correct => "bg-success",
                        LetterState::WrongPlace => "bg-warning",
                        LetterState::Wrong => "bg-secondary",
                        LetterState::Empty => "bg-light",
                    };
                    div class={"p-2 text-white border " (class)} { (letter.id) }
                }
            }
        }
    }
}

/// The markup for the text that the player is typing (uses alpine)
fn dynamic_word_markup() -> Markup {
    html! {
        div class="d-flex justify-content-center gap-1 mb-1" {
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
