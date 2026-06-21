use axum::{Router, Form, routing::get, response::{Html, IntoResponse, Redirect}};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use askama::Template;
use serde::Deserialize;
use crate::{app::AppState, auth::user::{UnauthenticatedUser, User}, error::AppError, repository::Repository};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", get(logout))
}

#[derive(Deserialize)]
struct CredentialsForm { username: String, password: String }

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage;

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn dashboard() -> Html<&'static str> {
    Html("TODO: dashboard")
}

/// Exibe a página de login. Redireciona para / se já autenticado.
async fn login_page(user: Option<User>) -> Result<impl IntoResponse, AppError> {
    if user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    Ok(Html(LoginPage.render()?).into_response())
}

/// Exibe a página de registro. Redireciona para / se já autenticado.
async fn register_page(user: Option<User>) -> Result<impl IntoResponse, AppError> {
    if user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    Ok(Html(RegisterPage.render()?).into_response())
}

async fn login(repository: Repository, jar: CookieJar, Form(form): Form<CredentialsForm>) -> Result<impl IntoResponse, AppError> {
    let unauth = UnauthenticatedUser::new(form.username, form.password);
    let record = unauth.authenticate(&repository).await?;
    let user = User::new(record.id, record.username);
    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true).path("/").build();
    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn register(repository: Repository, jar: CookieJar, Form(form): Form<CredentialsForm>) -> Result<impl IntoResponse, AppError> {
    let unauth = UnauthenticatedUser::new(form.username, form.password);
    let record = unauth.register(&repository).await?;
    let user = User::new(record.id, record.username);
    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true).path("/").build();
    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove(Cookie::from("token")), Redirect::to("/login"))
}
