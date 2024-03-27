use maud::{html, Markup, DOCTYPE};

use crate::Perk;

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-bs-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Search (htmx)" }

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

pub(crate) fn search_form(perks: &Vec<Perk>) -> Markup {
    html! {
        div ."container p-3" {

            h1 {
                i."bi bi-feather" {} " Search (htmx)"
            }

            div .card {
                div .card-header { "All perks" }
                div .card-body {
                    div ."d-flex justify-content-center flex-wrap gap-1" {
                        @for perk in perks {
                            div ."badge text-bg-light border" {
                                img src=(perk.icon) style="width: 25px; height: 25px;" .me-1; (perk.name)
                            }
                        }
                    }
                }
            }

            input
            ."form-control my-2"
            hx-trigger="input changed delay:500ms, search"
            type="search"
            name="term"
            hx-post="/search"
            hx-target="#search-results"
            hx-indicator=".htmx-indicator"
            placeholder="Begin Typing To Search Perks...";

            div ."htmx-indicator text-center" {
                small { i."bi bi-arrow-clockwise" {} " Searching..." }
            }

            table ."table" {
                thead {
                    tr {
                        th {
                            i."bi bi-image" {}
                        }
                        th {
                            "Name"
                        }
                        th {
                            "Description"
                        }
                    }
                }
                tbody #search-results {}
            }
        }
    }
}

pub(crate) fn search_perk_rows(perks: Vec<&Perk>) -> Markup {
    html! {
        @for perk in perks {
            tr {
                td { img src=(perk.icon) style="width: 25px; height: 25px;"; }
                td { (perk.name) }
                td { (perk.desc) }
            }
        }
    }
}
