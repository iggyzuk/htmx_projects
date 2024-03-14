use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::{
    constants::{CARD_ROUTER_CONTENT, CARD_ROUTER_CONTENT_ID, ROUTER_CONTENT, ROUTER_CONTENT_ID},
    state::{Ability, Hero, Talent},
};

/// Base HTML for new kind of a page.
/// e.g. `[main]` is used for all routing.
pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Routing (htmx)" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}
                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body {
                (content)
            }
        }
    }
}

/// A fragment that draws a bootstrap icon with some text.
pub(crate) fn icon_text<T: AsRef<str>>(icon: T, text: T, tinify: bool) -> Markup {
    let text = text.as_ref();
    html! { (self::icon(icon)) " " span .d-none[tinify] .d-sm-inline[tinify] { (text) } }
}

/// A fragment that draws a bootstrap icon.
pub(crate) fn icon<T: AsRef<str>>(icon: T) -> Markup {
    let icon = icon.as_ref();
    html! { i class={"bi bi-"(icon)} {} }
}

/// Main router content page.
/// Note that it includes the base.
pub(crate) fn router(content: Markup) -> Markup {
    base(html! {
        (navbar())
        hr;
        .container {
            // all child links will target the #router-content (using inheritance)
            div
            id=(ROUTER_CONTENT)
            hx-target=(ROUTER_CONTENT_ID)
            hx-boost="true"
            { (content) }
        }
        hr;
        div ."text-center m-3" { i { "© 2024 The Internet" } }
    })
}

pub(crate) fn home() -> Markup {
    let htmx = PreEscaped("<a href='http://htmx.org' target='_blank'>Htmx</a>");
    let rust = PreEscaped("<a href='https://www.rust-lang.org' target='_blank'>Rust</a>");
    let axum = PreEscaped("<a href='https://github.com/tokio-rs/axum' target='_blank'>Axum</a>");
    let maud = PreEscaped("<a href='https://maud.lambda.xyz' target='_blank'>Maud</a>");
    let bootstrap = PreEscaped("<a href='https://getbootstrap.com' target='_blank'>Bootstrap</a>");
    let iggyzuk = PreEscaped("<a href='http://iggyzuk.com' target='_blank'>Iggy Zuk</a>");

    html! {
        h1 { "Welcome!" }

        p { "This website demonstrates seamless routing using htmx. It supports responses as fragments and full pages. It reads " b .font-monospace { "hx-request" } " from the headers of the request to decide whether to send a fragment or a full page." }

        p { "For example, if you go to: " a href="/heroes" { "/heroes" } ", only the router content changes – this div with the text you're reading now. The header and the footer stay the same. Take a look at the network responses." }

        p { "To make things more interesting, there's a sub-router inside the hero page." }

        p { "For example, going to: " a href="/kelthuzad/talents" hx-target=(ROUTER_CONTENT_ID) { "/kelthuzad/talents" } " will auto-select the talents tab. If you change the tab and refresh the page, the full page will be sent back with the correct tab selected." }

        small { "Made with "(htmx)", "(rust)", "(axum)", "(maud)", and "(bootstrap)" by "(iggyzuk) }
    }
}

/// Navbar fragment that is part of the main layout.
/// It could be part of main, but this way it's cleaner.
pub(crate) fn navbar() -> Markup {
    html! {
        // note: buttons in nav-bar target the #router-content (using inheritance)
        nav."navbar navbar-expand-lg bg-body-tertiary" hx-target=(ROUTER_CONTENT_ID) hx-push-url="true" hx-boost="true" {
            div."container-fluid" {

                a."navbar-brand" href="/"
                { (icon_text("transparency", "Router", false)) span ."d-none d-md-inline" { " (htmx)" } }

                div .d-flex .justify-content-center .gap-2 {

                    button
                    hx-get="/heroes"
                    ."btn btn-outline-primary"
                    { (icon_text("emoji-smile-fill", "Heroes", true)) }

                    button
                    ."btn btn-outline-primary"
                    hx-get="/abilities"
                    { (icon_text("dpad-fill", "Abilities", true)) }

                    button
                    ."btn btn-outline-primary"
                    hx-get="/talents"
                    { (icon_text("star-fill", "Talents", true)) }

                    button
                    ."btn btn-outline-primary"
                    hx-get="/about"
                    { (icon_text("info-circle", "About", true)) }
                }
            }
        }
    }
}

/// A fragment that lists all heroes.
pub(crate) fn heroes(heroes: &[Hero]) -> Markup {
    html! {
        h1 { "Heroes" }
        ul .list-group {
            @for hero in heroes {
                a
                href={"/"(hero.id)}
                .list-group-item .list-group-item-action
                {
                    div ."d-flex justify-content-between" {
                        span {
                            img src=(hero.icon) style="width:25px;height:25px;" ."rounded me-2";
                            (hero.name)
                            span .text-secondary { " (" small { (hero.id) } ")" }
                        }
                        span {
                            span ."badge text-bg-danger d-none d-sm-inline" {
                                small { (hero.universe) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A fragment that lists all abilities.
pub(crate) fn abilities(abilities: Vec<&Ability>) -> Markup {
    html! {
        h1 { "Abilities" }
        (list_abilities(abilities))
    }
}

/// A fragment that lists all talents.
pub(crate) fn talents(talents: Vec<&Talent>) -> Markup {
    let mut talents = talents.clone();
    talents.sort_by_key(|x| x.level);
    html! {
        h1 { "Talents" }
        (list_talents(talents))
    }
}

/// A fragment for the about page.
/// Just something else to route to.
pub(crate) fn about() -> Markup {
    html! {
        h1 { "About" }
        p { "A routing example using htmx" }
        a href="/" class="btn btn-primary"{ "Go home" }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum HeroCardTab {
    Description,
    Abilities,
    Talents,
}

/// A reusable fragment that hightlights the current tab.
pub(crate) fn card_tab(tab: HeroCardTab, hero_id: &String) -> Markup {
    html! {
        div .card-header {
            ul .nav .nav-tabs .card-header-tabs {
                li .nav-item {
                    a .nav-link .active[tab == HeroCardTab::Description] href={"/"(hero_id)"/description"} { (icon_text("file-earmark-text-fill", "Description", true)) }
                }
                li .nav-item {
                    a .nav-link .active[tab == HeroCardTab::Abilities] href={"/"(hero_id)"/abilities"} { (icon_text("dpad-fill", "Abilities", true)) }
                }
                li .nav-item {
                    a .nav-link .active[tab == HeroCardTab::Talents] href={"/"(hero_id)"/talents"} { (icon_text("star-fill", "Talents", true)) }
                }
            }
        }
    }
}

/// A fragment that displays a hero.
/// It has an image and a sub-router for description, abilities, and talents.
pub(crate) fn hero(hero: &Hero, content: Markup) -> Markup {
    html! {
        div {
            div ."text-center" {
                h1 { (hero.name) }
                p .text-secondary { (hero.universe) }
            }
        }

        img ."img-fluid d-block mx-auto rounded" src=(hero.icon);

        // This is the card-router
        div
        id=(CARD_ROUTER_CONTENT)
        ."card text-center m-3"
        hx-target=(CARD_ROUTER_CONTENT_ID)
        { (content) }
    }
}

/// A fragment for the hero's description to be used within the sub-router.
pub(crate) fn hero_description(hero: &Hero) -> Markup {
    html! {
        (card_tab(HeroCardTab::Description, &hero.id))
        div .card-body {
            (hero.desc)
        }
    }
}

/// A fragment for the hero's abilities to be used within the sub-router.
pub(crate) fn hero_abilities(hero_id: &String, abilities: Vec<&Ability>) -> Markup {
    html! {
        (card_tab(HeroCardTab::Abilities, hero_id))
        // note: clicking on any ability will change the main '#rounter-content'
        // instead of the inherited '#card-router-content'
        div .card-body hx-target=(ROUTER_CONTENT_ID) {
            (list_abilities(abilities))
        }
    }
}

/// A fragment for the hero's talents to be used within the sub-router.
pub(crate) fn hero_talents(hero_id: &String, talents: Vec<&Talent>) -> Markup {
    html! {
        (card_tab(HeroCardTab::Talents, hero_id))
        // note: clicking on any ability will change the main '#rounter-content'
        // instead of the inherited '#card-router-content'
        div .card-body hx-target=(ROUTER_CONTENT_ID) {
            (list_talents(talents))
        }
    }
}

/// A fragment that draws a list of abilities.
pub(crate) fn list_abilities(abilities: Vec<&Ability>) -> Markup {
    html! {
        ul .list-group {
            @for ability in abilities {
                a
                href={"/"(ability.hero_id)"/ability/"(ability.id)}
                .list-group-item .list-group-item-action
                {
                    div ."d-flex justify-content-between" {
                        span {
                            (icon_text("dpad", &ability.name, false))
                        }
                        span ."badge text-bg-primary d-none d-sm-inline" {
                            small { (ability.hero_id) }
                        }
                    }
                }
            }
        }
    }
}

/// A fragment that draws a list of talents.
pub(crate) fn list_talents(talents: Vec<&Talent>) -> Markup {
    html! {
        ul .list-group {
            @for talent in talents {
                a
                href={"/"(talent.hero_id)"/talent/"(talent.id)}
                .list-group-item .list-group-item-action
                {
                    div ."d-flex justify-content-between" {
                        span {
                            (icon_text("stars", &talent.name, false))
                            span .text-secondary { " [" }
                            (talent_color(talent.level))
                            span .text-secondary { "]" }
                        }
                        span ."badge text-bg-warning d-none d-sm-inline" {
                            small { (talent.hero_id) }
                        }
                    }
                }
            }
        }
    }
}

/// A fragment for the ability details page.
pub(crate) fn ability(ability: &Ability) -> Markup {
    html! {
        h1 { (ability.name) }
        small .text-secondary { (ability.id)": " a href={"/"(ability.hero_id)} { (ability.hero_id)} }
        p { (ability.desc) }
    }
}

/// A fragment for the talent details page.
pub(crate) fn talent(talent: &Talent) -> Markup {
    html! {
        h1 { (talent.name) }
        small .text-secondary { (talent.id)" (level: "(talent_color(talent.level))"): " a href={"/"(talent.hero_id)} { (talent.hero_id)} }
        p { (talent.desc) }
    }
}

/// A fragment that draws the talent's level with a nice color.
pub(crate) fn talent_color(level: u8) -> Markup {
    let color = match level {
        1 => "#5E409D",
        4 => "#205EA6",
        7 => "#24837B",
        10 => "#66800B",
        13 => "#AD8301",
        16 => "#BC5215",
        20 => "#AF3029",
        _ => "#000000",
    };

    html! {
        span style={"color:"(color)";"} { (level) }
    }
}
