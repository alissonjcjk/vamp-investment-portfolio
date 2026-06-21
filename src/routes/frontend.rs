use axum::{Router, Form, routing::get, response::{Html, IntoResponse, Redirect}};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use askama::Template;
use serde::Deserialize;
use crate::{app::AppState, auth::user::{UnauthenticatedUser, User}, error::AppError, models::Asset, repository::Repository};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", get(logout))
}

#[derive(Deserialize)]
struct CredentialsForm { username: String, password: String }

// ── View models ──────────────────────────────────────────────────────────────

struct AssetView {
    id: i64,
    name: String,
    short_name: String,
    quantity: f64,
    unit_value: f64,
    #[allow(dead_code)]
    total_value: f64,
    percentage: f64,
    // Pre-formatted strings for the template
    quantity_fmt: String,
    unit_value_fmt: String,
    total_value_fmt: String,
    percentage_fmt: String,
}

fn fmt_brl(value: f64) -> String {
    // Format with 2 decimal places and Brazilian locale (comma as decimal separator)
    let s = format!("{:.2}", value);
    // Split integer and decimal parts
    let (int_part, dec_part) = s.split_once('.').unwrap_or((&s, "00"));
    // Add thousand separators
    let int_fmt: String = int_part
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(".");
    format!("{},{}", int_fmt, dec_part)
}

fn fmt_qty(value: f64) -> String {
    // Up to 4 decimal places, trimming trailing zeros
    let s = format!("{:.4}", value);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    username: String,
    total_value_fmt: String,
    assets: Vec<AssetView>,
    has_assets: bool,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage;

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn dashboard(user: Option<User>, repository: Repository) -> Result<impl IntoResponse, AppError> {
    let user = match user {
        Some(u) => u,
        None => return Ok(Redirect::to("/login").into_response()),
    };

    let assets = repository.list_assets_by_user(user.id()).await?;
    let total_value: f64 = assets.iter().map(Asset::total_value).sum();
    let has_assets = !assets.is_empty();

    let asset_views: Vec<AssetView> = assets.iter().map(|a| {
        let tv = a.total_value();
        let pct = if total_value > 0.0 { (tv / total_value) * 100.0 } else { 0.0 };
        let short_name_str = if a.name.chars().count() > 2 {
            a.name.chars().take(2).collect::<String>().to_uppercase()
        } else {
            a.name.clone().to_uppercase()
        };

        AssetView {
            id: a.id,
            name: a.name.clone(),
            short_name: short_name_str,
            quantity: a.quantity,
            unit_value: a.unit_value,
            total_value: tv,
            percentage: pct,
            quantity_fmt: fmt_qty(a.quantity),
            unit_value_fmt: fmt_brl(a.unit_value),
            total_value_fmt: fmt_brl(tv),
            percentage_fmt: format!("{:.1}", pct),
        }
    }).collect();

    let html = DashboardPage {
        username: user.username().to_string(),
        total_value_fmt: fmt_brl(total_value),
        assets: asset_views,
        has_assets,
    }.render()?;
    Ok(Html(html).into_response())
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
