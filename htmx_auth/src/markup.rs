use maud::{html, Markup, DOCTYPE};

use crate::User;

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-bs-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Auth (htmx)" }

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

pub(crate) fn home(user: &Option<User>) -> Markup {
    base(html! {
        div ."container d-flex justify-content-center p-2" {
            div style="max-width:400px;" {
                div .card {
                    div .card-body {

                        h1 .card-title { i ."bi bi-file-lock2" { } " OAuth2 (htmx)" }

                        p .card-text {
                            "Experience seamless Google OAuth2 authentication with HTMX, reinforced by server-side sessions for CSRF protection."
                        }

                        p { small .card-text .text-secondary { "Made by " a href="https://iggyzuk.com/" { "Iggy Zuk" } } }

                        div #protected-content hx-get="/protected-fragment" hx-trigger="load" { }

                        div ."text-center" {
                            @if user.is_none() {
                                a
                                href="/auth/google"
                                ."btn btn-lg btn-primary w-100"
                                { i ."bi bi-patch-check-fill" { } " Continue with Google" }
                            }
                        }

                    }
                }
            }
        }
    })
}

pub(crate) fn protected(user: &User) -> Markup {
    html! {
        div ."text-center border rounded m-2 p-2" {
            p .text-secondary { small { i ."bi bi-file-person-fill" { } " " (user.id) } }
            img src=(user.picture) .rounded;
            h2 { (user.name) }
            h3 { (user.email) }
            @if user.verified_email {
                p .text-success small { { i ."bi bi-envelope-at-fill" { } " Verified Email" } }
            }
            a
            href="/logout"
            hx-boost="true"
            ."btn btn-lg btn-outline-danger w-100"
            { i ."bi bi-x-square-fill" { } " Logout" }
        }
    }
}
