use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::Markup;

use crate::{constants, hx::HxReq, markup, state::AppState};

pub(crate) async fn index(hx_req: HxReq) -> Markup {
    let fragment = markup::home();
    if hx_req.is_targeting(constants::ROUTER_CONTENT) {
        fragment
    } else {
        markup::router(fragment)
    }
}

pub(crate) async fn heroes(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    let heroes = state.heroes.read().await;
    let fragment = markup::heroes(&heroes.0);
    if hx_req.is_targeting(constants::ROUTER_CONTENT) {
        fragment
    } else {
        markup::router(fragment)
    }
}

pub(crate) async fn abilities(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    let abilities = state.abilities.read().await;
    let fragment = markup::abilities(abilities.0.iter().collect::<Vec<_>>());
    if hx_req.is_targeting(constants::ROUTER_CONTENT) {
        fragment
    } else {
        markup::router(fragment)
    }
}

pub(crate) async fn talents(State(state): State<Arc<AppState>>, hx_req: HxReq) -> Markup {
    let talents = state.talents.read().await;
    let fragment = markup::talents(talents.0.iter().collect::<Vec<_>>());
    if hx_req.is_targeting(constants::ROUTER_CONTENT) {
        fragment
    } else {
        markup::router(fragment)
    }
}

pub(crate) async fn hero(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    hx_req: HxReq,
) -> Response {
    let heroes = state.heroes.read().await;
    let hero = heroes.read(id);

    if let Some(hero) = hero {
        let fragment = markup::hero(hero, markup::hero_description(hero));
        if hx_req.is_targeting(constants::ROUTER_CONTENT) {
            fragment
        } else {
            markup::router(fragment)
        }
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
        let fragment = markup::hero_description(hero);
        if hx_req.is_targeting(constants::CARD_ROUTER_CONTENT) {
            fragment
        } else {
            markup::router(markup::hero(hero, fragment))
        }
        .into_response()
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
        let fragment =
            markup::hero_abilities(&hero.id, state.abilities.read().await.for_hero(&hero.id));

        if hx_req.is_targeting(constants::CARD_ROUTER_CONTENT) {
            fragment
        } else {
            markup::router(markup::hero(hero, fragment))
        }
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
        let fragment =
            markup::hero_talents(&hero.id, state.talents.read().await.for_hero(&hero.id));
        if hx_req.is_targeting(constants::CARD_ROUTER_CONTENT) {
            fragment
        } else {
            markup::router(markup::hero(hero, fragment))
        }
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
        if hx_req.is_targeting(constants::ROUTER_CONTENT) {
            markup::ability(&ability)
        } else {
            markup::router(markup::ability(&ability))
        }
        .into_response()
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
        if hx_req.is_targeting(constants::ROUTER_CONTENT) {
            markup::talent(&talent)
        } else {
            markup::router(markup::talent(&talent))
        }
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "talent doesn't exist").into_response()
    }
}

pub(crate) async fn about(hx_req: HxReq) -> Markup {
    if hx_req.is_targeting(constants::ROUTER_CONTENT) {
        markup::about()
    } else {
        markup::router(markup::about())
    }
}
