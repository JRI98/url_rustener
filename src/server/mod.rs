mod database;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_garde::WithValidation;
use garde::Validate;
use nanoid::nanoid;
use serde::Deserialize;
use tracing::{error, warn};

use self::database::KVDatabase;

const SLUG_LENGTH: usize = 21;

#[derive(Clone)]
pub struct AppState {
    db: Arc<KVDatabase>,
}

impl AppState {
    pub fn new(redis_url: &str) -> Result<AppState> {
        Ok(AppState {
            db: Arc::new(KVDatabase::new(redis_url)?),
        })
    }
}

impl axum::extract::FromRef<AppState> for () {
    fn from_ref(_: &AppState) {}
}

#[derive(Deserialize, Validate)]
pub struct SlugPath(#[garde(length(min=SLUG_LENGTH, max=SLUG_LENGTH))] String);

#[derive(Deserialize, Validate)]
pub struct KeyQuery {
    #[garde(length(max = 32))]
    key: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateURLBody {
    #[garde(length(max = 32))]
    key: String,

    #[garde(url)]
    url: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateURLBody {
    #[garde(length(max = 32))]
    key: String,
}

pub struct Routes;

impl Routes {
    pub async fn get_url(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
    ) -> Response {
        if let Err(err) = state.db.hincrby(&path.0, "stats", 1) {
            warn!("get_url: {}", err); // This is not critical
        }

        match state.db.hget(&path.0, "url") {
            Ok(None) => Redirect::permanent("/").into_response(),
            Ok(Some(url)) => Redirect::permanent(&url).into_response(),
            Err(err) => {
                error!("get_url: {}", err);
                Redirect::permanent("/").into_response()
            }
        }
    }

    pub async fn get_url_stats(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
        WithValidation(query): WithValidation<Query<KeyQuery>>,
    ) -> Response {
        let key = match state.db.hget(&path.0, "key") {
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Ok(Some(key)) => key,
            Err(err) => {
                error!("get_url_stats: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.hget(&path.0, "stats") {
            Ok(stats) => Json(stats).into_response(),
            Err(err) => {
                error!("get_url_stats: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    pub async fn create_url(
        State(state): State<AppState>,
        WithValidation(payload): WithValidation<Json<CreateURLBody>>,
    ) -> Response {
        let slug = nanoid!(SLUG_LENGTH);

        match state
            .db
            .hset(&slug, "url", &payload.url)
            .and_then(|_| state.db.hset(&slug, "key", &payload.key))
        {
            Ok(()) => (StatusCode::CREATED, slug).into_response(),
            Err(err) => {
                error!("create_url: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    pub async fn update_url(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
        WithValidation(query): WithValidation<Query<KeyQuery>>,
        WithValidation(payload): WithValidation<Json<UpdateURLBody>>,
    ) -> Response {
        let key = match state.db.hget(&path.0, "key") {
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Ok(Some(key)) => key,
            Err(err) => {
                error!("update_url: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.hset(&path.0, "key", &payload.key) {
            Ok(()) => "Successfully update URL".into_response(),
            Err(err) => {
                error!("update_url: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    pub async fn delete_url(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
        WithValidation(query): WithValidation<Query<KeyQuery>>,
    ) -> Response {
        let key = match state.db.hget(&path.0, "key") {
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Ok(Some(key)) => key,
            Err(err) => {
                error!("delete_url: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.del(&path.0) {
            Ok(()) => "Successfully deleted URL".into_response(),
            Err(err) => {
                error!("delete_url: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
