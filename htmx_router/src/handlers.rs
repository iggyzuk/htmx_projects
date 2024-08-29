use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::Markup;

use crate::{
    constants::{CARD_ROUTER_CONTENT, ROUTER_CONTENT},
    hx::HxReq,
    markup,
    state::{AppState, Hero},
};

pub(crate) async fn index(hx_req: HxReq) -> Markup {
    router_fragment_stack(hx_req, markup::home())
}

pub(crate) async fn heroes(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    router_fragment_stack(hx_req, markup::heroes(&state.heroes.read().await.0))
}

pub(crate) async fn abilities(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    router_fragment_stack(
        hx_req,
        markup::abilities(state.abilities.read().await.0.iter().collect::<Vec<_>>()),
    )
}

pub(crate) async fn talents(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    router_fragment_stack(
        hx_req,
        markup::talents(state.talents.read().await.0.iter().collect::<Vec<_>>()),
    )
}

pub(crate) async fn hero(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    hx_req: HxReq,
) -> Response {
    let heroes = state.heroes.read().await;
    let hero = heroes.read(id);

    if let Some(hero) = hero {
        router_fragment_stack(hx_req, markup::hero(hero, markup::hero_description(hero)))
            .into_response()
    } else {
        (StatusCode::NOT_FOUND, "hero doesn't exist").into_response()
    }
}

pub(crate) async fn hero_description(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    hx_req: HxReq,
) -> Response {
    let heroes = state.heroes.read().await;
    let hero = heroes.read(id);

    if let Some(hero) = hero {
        hero_fragment_stack(hx_req, hero, markup::hero_description(hero)).into_response()
    } else {
        (StatusCode::NOT_FOUND, "hero doesn't exist").into_response()
    }
}

pub(crate) async fn hero_abilities(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    hx_req: HxReq,
) -> Response {
    let heroes = state.heroes.read().await;
    let hero = heroes.read(id);

    if let Some(hero) = hero {
        hero_fragment_stack(
            hx_req,
            hero,
            markup::hero_abilities(&hero.id, state.abilities.read().await.for_hero(&hero.id)),
        )
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "hero doesn't exist").into_response()
    }
}

pub(crate) async fn hero_talents(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    hx_req: HxReq,
) -> Response {
    let heroes = state.heroes.read().await;
    let hero = heroes.read(id);

    if let Some(hero) = hero {
        hero_fragment_stack(
            hx_req,
            hero,
            markup::hero_talents(&hero.id, state.talents.read().await.for_hero(&hero.id)),
        )
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "hero doesn't exist").into_response()
    }
}

pub(crate) async fn ability(
    State(state): State<Arc<AppState>>,
    Path((_hero_id, ability_id)): Path<(String, String)>,
    hx_req: HxReq,
) -> Response {
    let abilities = state.abilities.read().await;
    if let Some(ability) = abilities.read(ability_id) {
        router_fragment_stack(hx_req, markup::ability(ability)).into_response()
    } else {
        (StatusCode::NOT_FOUND, "ability doesn't exist").into_response()
    }
}

pub(crate) async fn talent(
    State(state): State<Arc<AppState>>,
    Path((_hero_id, ability_id)): Path<(String, String)>,
    hx_req: HxReq,
) -> Response {
    let talents = state.talents.read().await;
    if let Some(talent) = talents.read(ability_id) {
        router_fragment_stack(hx_req, markup::talent(talent)).into_response()
    } else {
        (StatusCode::NOT_FOUND, "talent doesn't exist").into_response()
    }
}

pub(crate) async fn about(hx_req: HxReq) -> Markup {
    router_fragment_stack(hx_req, markup::about())
}

/// This functions returns the fragment as is when the target matches otherwise
/// it send the full page.
fn router_fragment_stack(hx_req: HxReq, fragment: Markup) -> Markup {
    if hx_req.is_targeting(ROUTER_CONTENT) {
        fragment
    } else {
        markup::router(fragment)
    }
}

/// This function works with main-router and sub-router.
/// It can return fragments with various level of completeness depending on the
/// target of the request.
///
/// # Example:
/// 1. Request targets the sub-router (the inner tab inside the hero page) then
///    we return the fragment as is.
/// 2. Request targets the main-router (e.g. hero page) then we need to wrap it
///    like an onion with the hero page markup: hero -> talents.
/// 3. Request doesn't target anything, it means we need to send the full body:
///    router -> hero -> talents.
fn hero_fragment_stack(hx_req: HxReq, hero: &Hero, fragment: Markup) -> Markup {
    if hx_req.is_targeting(CARD_ROUTER_CONTENT) {
        fragment
    } else if hx_req.is_targeting(ROUTER_CONTENT) {
        markup::hero(hero, fragment)
    } else {
        markup::router(markup::hero(hero, fragment))
    }
}
