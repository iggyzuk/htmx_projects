use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Form,
};
use maud::{html, Markup, Render, DOCTYPE};
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
        tasks.push(Task::new("Add the ability to edit these tasks".to_string()));
        tasks.push(Task::new("Make this crud experiement prettier".to_string()));
        tasks.push(Task::new("Take over the world".to_string()));
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
    fn update(&mut self, id: Uuid) {
        if let Some(task) = self.0.iter_mut().find(|t| t.id == id) {
            task.complete = !task.complete;
        }
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

    let app = axum::Router::new()
        .route("/", get(index))
        .route("/task", post(create_task))
        .route("/task/:id", get(read_task))
        .route("/task/:id", put(update_task))
        .route("/task/:id", delete(delete_task))
        .route("/task/:id/edit", get(get_edit_task))
        .route("/tasks", get(tasks))
        .layer(cors)
        .with_state(AppState::new());

    let address = "0.0.0.0:4242";

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "CRUD in HTMX" }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body {
                .container {
                    .card .m-3 {
                        h5 .card-header { "CRUD in HTMX" }
                        .card-body {
                            // main form
                            form hx-post="/task" hx-target="#task-list" {
                                div class="input-group mb-3" {
                                    // task name input
                                    input
                                    id="title"
                                    name= "title"
                                    type="text"
                                    class="form-control"
                                    placeholder="What would you like to do?"
                                    aria-label="Task name"
                                    aria-describedby="button-addon2"
                                    {}
                                    // submit button
                                    button type="submit" class="btn btn-outline-secondary" id="button-addon2" { "Create Task" }
                                }
                            }
                            // all tasks
                            div id="task-list" hx-get="/tasks" hx-trigger="load" class="card" {}
                        }
                    }
                }
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

async fn read_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let tasks = state.tasks.read().await;
    let task = tasks.read(id);

    if let Some(task) = task {
        return task.render().into_response();
    }
    (StatusCode::GONE, "task doesn't exist").into_response()
}

async fn update_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Markup {
    let mut tasks = state.tasks.write().await;
    tasks.update(id);
    tasks.render()
}

async fn delete_task(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Markup {
    let mut tasks = state.tasks.write().await;
    tasks.delete(id);
    tasks.render()
}

async fn tasks(State(state): State<Arc<AppState>>) -> Markup {
    state.tasks.read().await.render()
}

// todo: replace task title with edit + add save/cancel buttons
async fn get_edit_task(State(state): State<Arc<AppState>>) -> Markup {
    html! {
        "Edit"
    }
}

impl Render for Tasks {
    fn render(&self) -> Markup {
        html! {
            ul class="list-group list-group-flush" {
                @for task in self.0.iter() {
                    li .d-flex .justify-content-between .align-items-center .list-group-item {
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
            // check combo
            div class="form-check form-switch" {

                // complete: input (checkbox)
                input
                type="checkbox"
                role="switch"
                id={"task_"(self.id)}
                .form-check-input
                checked[self.complete]
                hx-put={"/task/"(self.id)}
                hx-trigger="click"
                hx-target="#task-list"
                {}

                // title: label
                label
                .form-check-label
                for={"task_"(self.id)}
                { (self.title) }
            }

            // delete: button
            button
            .btn .btn-danger
            hx-delete={"/task/"(self.id)}
            hx-trigger="click"
            hx-target="#task-list"
            { "Delete" }
        }
    }
}
