use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::state::Image;

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-bs-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Images (htmx)" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}

                script src="https://unpkg.com/htmx.org@1.9.10" {}
                // script src="https://unpkg.com/hyperscript.org@0.9.12" {}

                style { (global_loading_bar_style()) }
            }

            body hx-indicator=".loading-bar" {
                div ."loading-bar" {}
                (content)
            }
        }
    }
}

pub(crate) fn global_loading_bar_style() -> &'static str {
    r#"
.loading-bar {
    opacity: 0;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 2px;
    background: linear-gradient(90deg, transparent,
        #ffc107, transparent,
        #ffc107, transparent
    );
}

.htmx-request.loading-bar {
    opacity: 1;
    animation: fadeIn 0.2s linear forwards, slide 0.8s ease-in-out infinite;
}

@keyframes slide {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX( 100%); }
}

@keyframes fadeIn {
    0%   { opacity: 0; }
    50%  { opacity: 0; }
    100% { opacity: 1; }
}
"#
}

pub(crate) fn home() -> Markup {
    let content = html! {
        div ."container-flex m-3" {
            (form())

            // images appear here
            div
            #all-images
            ."d-flex justify-content-center flex-wrap gap-1"
            hx-trigger="load" hx-get="/images"
            { h1 { "💿" } }
        }
    };
    base(content)
}

pub(crate) fn form() -> Markup {
    let script = r##"
        htmx.on('#img-upload-form', 'htmx:xhr:progress', function(evt) {
            var loaded = evt.detail.loaded;
            var total = evt.detail.total;
            var progress = (loaded / total) * 100;

            document.getElementById('progress').style.width = progress + '%';
        });

        htmx.on('htmx:afterRequest', function(evt) {
            if (evt.detail.elt && evt.detail.elt.id === 'img-upload-form') {
                document.getElementById('progress').style.width = '0%'; // Reset progress bar
                document.getElementById('form-file').value = ''; // Clear file input field
            }
        });
    "##;

    html! {
        div {
            h1 { i ."bi bi-file-image" {} " Images" }
            hr;
            div .m-2 {

                form
                #img-upload-form
                .row
                hx-encoding="multipart/form-data"
                hx-post="/images"
                hx-target="#all-images"
                hx-swap="afterbegin"
                hx-disabled-elt="#sub-btn"
                // You can do the progress bar animation with hyperscript too, but how about the rest?
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
        script { (PreEscaped(script)) }
    }
}

pub(crate) fn image(img: &Image) -> Markup {
    html! {
        div style="max-width:150px;" {
            a href={"/images/"(img.id)} { img .rounded src=(img.src()); }
            div .text-truncate .fw-bold { (img.file_name) }
            small .text-wrap { (img.id) ": " (img.short_date()) }
        }
    }
}

pub(crate) fn images(images: &Vec<Image>) -> Markup {
    html! {
        @for image in images {
            (self::image(image))
        }
    }
}