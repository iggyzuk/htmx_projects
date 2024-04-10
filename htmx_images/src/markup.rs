use maud::{html, Markup, DOCTYPE};

use crate::state::Image;

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-bs-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Images (htmx)" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}

                script src="https://unpkg.com/htmx.org@1.9.10" {}
                // script src="https://unpkg.com/hyperscript.org@0.9.12" {}

                // custom css and scripts
                link rel="stylesheet" href="/assets/loading-bar.css";
                link rel="stylesheet" href="/assets/grid.css";
                script src="/assets/upload.js" {}
            }

            body hx-indicator=".loading-bar" {
                div ."loading-bar" {}
                (content)
            }
        }
    }
}

pub(crate) fn home() -> Markup {
    let content = html! {
        div ."container-flex m-3" {
            (form())

            // images appear here
            div
            #all-images
            hx-trigger="load" hx-get="/images"
            { h1 .text-center { "💿" } }

            (modal_base())
        }
    };
    base(content)
}

pub(crate) fn form() -> Markup {
    html! {
        div {
            h1 { i ."bi bi-file-image" {} " Images" }
            hr;
            div .my-2 {

                form
                #img-upload-form
                .row
                hx-encoding="multipart/form-data"
                hx-post="/images"
                hx-target="#all-images"
                hx-swap="afterbegin"
                hx-disabled-elt="#sub-btn"
                // # you can do the progress bar animation with hyperscript too, but how about the rest?
                // _="on htmx:xhr:progress(loaded, total) set *width of #progress to (((loaded/total)*100) + '%')"
                {
                    div ."col-sm-8 mb-2 mb-sm-0" {
                        input id="form-file" ."form-control form-control-lg" type="file" name="file" accept="image/jpeg, image/png, image/webp, image/gif";
                    }
                    div .col-sm-4 {
                        button #sub-btn ."btn btn-primary btn-lg w-100" { i ."bi bi-file-earmark-arrow-up-fill" {} " Upload" }
                    }
                }
                div ."progress mt-3" role="progressbar" {
                    div id="progress" class="progress-bar progress-bar-striped progress-bar-animated" style="width:0%;" {}
                }
            }
            hr;
        }
    }
}

pub(crate) fn image(img: &Image) -> Markup {
    html! {
        a
        ."grid-item"
        hx-get={"/images/"(img.id)"/modal"}
        hx-target="#modals-here"
        hx-trigger="click"
        data-bs-toggle="modal"
        data-bs-target="#modals-here"
        {
            img src=(img.src());
        }
    }
}

pub(crate) fn images(images: &Vec<Image>) -> Markup {
    html! {
        div .grid-container {
            @for image in images {
                (self::image(image))
            }
        }
    }
}

pub(crate) fn modal_base() -> Markup {
    html! {
        div
        #modals-here
        ."modal modal-blur fade"
        style="display: none"
        aria-hidden="false"
        tabindex="-1"
        {
            div
            ."modal-dialog modal-lg modal-dialog-centered"
            role="document"
            {
                div class="modal-content" {}
            }
        }
    }
}

pub(crate) fn image_modal(img: &Image) -> Markup {
    html! {
        div ."modal-dialog modal-dialog-centered" {
            div ."modal-content" {
                div ."modal-header" {
                    h5 ."modal-title text-truncate" { (img.file_name) }
                }
                div ."modal-body" {
                    img src=(img.src()) ."w-100";
                    small .text-wrap { (img.id) ": " (img.short_date()) }
                }
            }
        }
    }
}
