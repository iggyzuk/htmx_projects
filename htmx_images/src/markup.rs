use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::state::Image;

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Images (htmx)" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}

                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="https://unpkg.com/hyperscript.org@0.9.12" {}
            }
            body {
                (content)
            }
        }
    }
}

pub(crate) fn home() -> Markup {
    let content = html! {
        div ."container-flex m-3" {
            (form())
            div #all-images ."d-flex justify-content-center flex-wrap"
            hx-trigger="load" hx-get="/images" { "🔥" }
        }
        // (grid())
    };
    base(content)
}

// todo: how do you catch events and reset the progress bar when the request responds?
pub(crate) fn form() -> Markup {
    let script = PreEscaped(
        r##"
htmx.on('#img-upload-form', 'htmx:xhr:progress', function(evt) {
    var loaded = event.detail.loaded;
        var total = event.detail.total;
        var progress = (loaded / total) * 100;

        document.getElementById('progress').style.width = progress + '%';
});
    "##,
    );

    html! {

        div {

            h1 { "Images" }

            hr;

            form
            #img-upload-form
            hx-encoding="multipart/form-data" hx-post="/images" hx-target="#all-images" hx-swap="beforeend"

            // question: is this possible with hyperscript?
            // _="on htmx:xhr:progress(loaded, total) set #progress.value to (loaded/total)*100"
            {
                input type="file" name="file" accept="image/jpeg, image/png";
                button .btn .btn-warning { "Upload" }

                div class="progress m-3" role="progressbar"{
                    div id="progress" class="progress-bar progress-bar-striped progress-bar-animated" style="width:0%;" {}
                }
            }

            hr;
        }

        script { (script) }
    }
}

pub(crate) fn image(img: &Image) -> Markup {
    html! {
        img src=(img.src());
        // span { (img.id) ", " (img.file_name) ", " (img.mime_type) ", " (img.created_at) }
    }
}

pub(crate) fn images(images: &Vec<Image>) -> Markup {
    html! {
        @for image in images {
            (self::image(image))
        }
    }
}

// pub(crate) fn grid(images: Vec<&Image>) -> Markup {
//     let img_base64 = include_str!("../img-base64.txt");

//     html! {
//         style {
//             "
//             .box {
//                 min-width: 150px;
//                 max-width: 300px;
//                 height: 150px;
//                 object-fit: cover;
//                 cursor: pointer;
//             }
//             "
//         }
//         div."container-flex text-center p-3" {
//             div."row gap-1" {
//                 @for _i in 0..100 {
//                     // div."col bg-dark text-white box d-flex justify-content-center align-items-center" {
//                         // (i)
//                         img ."col bg-dark text-white box d-flex justify-content-center align-items-center p-0 rounded cursor-pointer" src=(img_base64);
//                     // }
//                 }
//             }
//         }
//     }
// }
