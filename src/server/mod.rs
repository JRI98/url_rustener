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
const KEY_MAX_LENGTH: usize = 64;

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
    #[garde(length(max = KEY_MAX_LENGTH))]
    key: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateURLBody {
    #[garde(length(max = KEY_MAX_LENGTH))]
    key: String,

    #[garde(url)]
    url: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateURLBody {
    #[garde(length(max = KEY_MAX_LENGTH))]
    key: String,
}

pub struct Routes;

impl Routes {
    pub async fn get_url(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
    ) -> Response {
        match state.db.hget(&path.0, "url") {
            Ok(Some(url)) => {
                if let Err(err) = state.db.hincrby(&path.0, "total_accesses", 1) {
                    warn!("get_url: {}", err); // This is not critical
                }
                Redirect::permanent(&url).into_response()
            }
            Ok(None) => StatusCode::NOT_FOUND.into_response(),
            Err(err) => {
                error!("get_url: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    pub async fn get_url_stats(
        State(state): State<AppState>,
        WithValidation(path): WithValidation<Path<SlugPath>>,
        WithValidation(query): WithValidation<Query<KeyQuery>>,
    ) -> Response {
        let key = match state.db.hget(&path.0, "key") {
            Ok(Some(key)) => key,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(err) => {
                error!("get_url_stats: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.hget(&path.0, "total_accesses") {
            Ok(Some(total_accesses)) => Json(total_accesses).into_response(),
            Ok(None) => StatusCode::NOT_FOUND.into_response(),
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
            Ok(_) => (StatusCode::CREATED, slug).into_response(),
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
            Ok(Some(key)) => key,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(err) => {
                error!("update_url: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.hset(&path.0, "key", &payload.key) {
            Ok(_) => StatusCode::OK.into_response(),
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
            Ok(Some(key)) => key,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(err) => {
                error!("delete_url: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if key != query.key {
            return StatusCode::UNAUTHORIZED.into_response();
        }

        match state.db.del(&path.0) {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => {
                error!("delete_url: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
