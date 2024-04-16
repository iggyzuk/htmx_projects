use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

struct AppState {
    tasks: RwLock<Tasks>,
}

impl AppState {
    fn new() -> Arc<Self> {
        let mut tasks = vec![];
        tasks.push(Task::new("Learn how to be epic with htmx".to_string()));
        tasks.push(Task::new_done(
            "Add the ability to edit these tasks".to_string(),
        ));
        tasks.push(Task::new_done(
            "Make this crud experiement prettier".to_string(),
        ));
        tasks.push(Task::new(
            "Take over the world (break it up in simpler tasks)".to_string(),
        ));
        tasks.push(Task::new_done("Become enlightened ✨".to_string()));

        Arc::new(AppState {
            tasks: RwLock::new(Tasks(tasks)),
        })
    }
}

struct Tasks(Vec<Task>);

impl Tasks {
    fn create(&mut self, title: String) {
        self.0.push(Task::new(title)); // todo: from form data
    }
    fn read(&self, id: Uuid) -> Option<&Task> {
        self.0.iter().find(|t| t.id == id)
    }
    fn read_mut(&mut self, id: Uuid) -> Option<&mut Task> {
        self.0.iter_mut().find(|t| t.id == id)
    }
    fn delete(&mut self, id: Uuid) {
        self.0.retain(|t| t.id != id);
    }
}

struct Task {
    id: Uuid,
    title: String,
    complete: bool,
}

impl Task {
    fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            complete: false,
        }
    }

    fn new_done(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            complete: true,
        }
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let task_routes = Router::new()
        .route("/", post(create_task))
        .route("/:id", get(read_task).put(update_task).delete(delete_task))
        .route("/:id/edit", get(get_edit_task).post(post_edit_task));

    let app = Router::new()
        .route("/", get(index))
        .nest("/task", task_routes)
        .route("/tasks", get(tasks))
        .layer(cors)
        .with_state(AppState::new());

    let address = "0.0.0.0:4203";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Markup {
    let scripts = PreEscaped(include_str!("../scripts.js"));
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "CRUD (htmx)" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="https://unpkg.com/htmx.org/dist/ext/disable-element.js" {}
                script { (scripts) }
            }
            body {
                .container {

                    .card .m-3 {
                        h5 .card-header { "Tasks" }
                        .card-body {
                            // main form to create tasks
                            form hx-post="/task" hx-target="#task-list" autocomplete="off" {
                                div class="input-group mb-3" {

                                    // task name input
                                    input
                                    id="title"
                                    name= "title"
                                    type="text"
                                    class="form-control"
                                    placeholder="What would you like to do?"
                                    aria-label="Task name"
                                    {}

                                    // submit button
                                    button
                                    type="submit"
                                    class="btn btn-outline-primary"
                                    { "Create Task" }
                                }
                            }
                            // all tasks
                            div id="task-list" hx-get="/tasks" hx-trigger="load" class="card" {}
                        }
                    }
                }

                (confirm_modal_markup())
            }
        }
    }
}

#[derive(Deserialize)]
struct CreateTaskForm {
    title: String,
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Form(query): Form<CreateTaskForm>,
) -> impl IntoResponse {
    let mut tasks = state.tasks.write().await;
    tasks.create(query.title);
    (StatusCode::CREATED, tasks.render())
}

async fn read_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Response {
    let tasks = state.tasks.read().await;
    let task = tasks.read(id);

    if let Some(task) = task {
        return task.render().into_response();
    }
    (StatusCode::NOT_FOUND, "task doesn't exist").into_response()
}

async fn update_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Response {
    let mut tasks = state.tasks.write().await;

    if let Some(task) = tasks.0.iter_mut().find(|t| t.id == id) {
        task.complete = !task.complete;
        return task.render().into_response();
    }
    (StatusCode::NOT_FOUND, "task doesn't exist").into_response()
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut tasks = state.tasks.write().await;
    tasks.delete(id);
    (StatusCode::OK, Body::empty())
}

async fn tasks(State(state): State<Arc<AppState>>) -> Markup {
    state.tasks.read().await.render()
}

async fn get_edit_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Response {
    let tasks = state.tasks.read().await;
    if let Some(task) = tasks.read(id) {
        return html! {
            form hx-post={"/task/"(task.id)"/edit"} hx-target={"#task_"(task.id)} hx-swap="outerHTML" autocomplete="off" .w-100  {

                div ."input-group" {

                    // Input value for modified task title
                    input
                    id="title"
                    name="title"
                    class="form-control"
                    placeholder="Title"
                    value=(task.title)
                    type="text"
                    aria-label="Text input with segmented dropdown button"
                    {}

                    // Save button (submits the form)
                    button
                    ."btn btn-outline-primary"
                    type="submit"
                    hx-ext="disable-element"
                    hx-disable-element="self"
                    { "Save" }

                    // Dropdown button for extra options
                    button ."btn btn-outline-primary dropdown-toggle dropdown-toggle-split"
                    data-bs-toggle="dropdown"
                    aria-expanded="false"
                    type="button" {
                        span ."visually-hidden" {
                            "Toggle Dropdown"
                        }
                    }

                    // Dropdown options
                    ul ."dropdown-menu dropdown-menu-end" {

                        // Cancel the edit (and show the old task)
                        li {
                            .dropdown-item 
                            type="button"
                            hx-get={"/task/"(task.id)}
                            hx-target={"#task_"(task.id)}
                            { "Cancel" }
                        }

                        // Delete the task (and update all tasks)
                        li {
                            ."dropdown-item text-danger"
                            type="button"
                            hx-trigger="click"
                            hx-confirm="Are you sure you want to delete this task?"
                            hx-delete={"/task/"(task.id)}
                            hx-target="closest .li-task"
                            hx-swap="delete"
                            { "Delete" }
                        }
                    }
                }
            }
        }.into_response();
    }
    (StatusCode::NOT_FOUND, "task doesn't exist").into_response()
}

#[derive(Deserialize)]
struct EditTaskForm {
    title: String,
}

async fn post_edit_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Form(query): Form<EditTaskForm>,
) -> Response {
    if let Some(task) = state.tasks.write().await.read_mut(id) {
        task.title = query.title;
        return task.render().into_response();
    }
    (StatusCode::NOT_FOUND, "task doesn't exist").into_response()
}

impl Render for Tasks {
    fn render(&self) -> Markup {
        html! {
            ul class="list-group list-group-flush" {
                @for task in self.0.iter() {
                    li .li-task .list-group-item {
                        (&task)
                    }
                }
            }
        }
    }
}

impl Render for Task {
    fn render(&self) -> Markup {
        html! {
            #{"task_"(self.id)} .d-flex .justify-content-between .align-items-center {
                // check combo
                div class="form-check form-switch" {

                    // complete: input (checkbox)
                    input
                    type="checkbox"
                    role="switch"
                    id={"task_"(self.id)"_input"}
                    .form-check-input
                    checked[self.complete]
                    hx-put={"/task/"(self.id)}
                    hx-trigger="click"
                    hx-target={"#task_"(self.id)}
                    hx-swap="outerHTML"
                    hx-ext="disable-element"
                    hx-disable-element="self"
                    {}

                    // title: label
                    label
                    id={"task_"(self.id)"_label"} // this can be made reusable
                    .form-check-label
                    for={"task_"(self.id)"_input"}
                    { (self.title) }
                }

                .d-flex .no-wrap {
                    // edit: button
                    button
                    .btn .btn-outline-warning .me-1
                    hx-get={"/task/"(self.id)"/edit"}
                    hx-trigger="click"
                    hx-target={"#task_"(self.id)}
                    hx-ext="disable-element"
                    hx-disable-element="self"
                    { "Edit" }
                }
            }
        }
    }
}

fn confirm_modal_markup() -> Markup {
    html! {
        div."modal fade" id="confirm-modal" role="dialog" aria-hidden="true" tabindex="-1" aria-labelledby="confirm-modal-label" {
            div."modal-dialog modal-dialog-centered" role="document" {
                div."modal-content" {
                    div."modal-header" {
                        // Title
                        h5."modal-title" id="confirm-modal-label" {
                            "Delete Task"
                        }
                        // X button to close the modal
                        button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" {}
                    }
                    // Are you sure you want to delete...
                    div."modal-body" {
                        p #confirm-modal-text {}
                    }
                    div."modal-footer" {
                        // Cancel delete
                        button."btn btn-secondary" #confirm-modal-cancel type="button" data-bs-dismiss="modal" {
                            "Cancel"
                        }
                        // Confirm delete
                        button."btn btn-danger" #confirm-modal-proceed type="button" {
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
