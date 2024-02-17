mod auth;
mod utilities;

use std::{collections::HashMap, sync::Arc};

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use auth::{
    auth_login, auth_redirect, oidc_client, oidc_provider_metadata, AuthState, AuthStateStore,
};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Extension, Router,
};
use nimble_graphql::QueryRoot;
use openidconnect::core::{CoreClient, CoreProviderMetadata};

use tokio::{net::TcpListener, sync::Mutex};
use tower_cookies::CookieManagerLayer;
use utilities::Cached;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let auth_state_store =
        AuthStateStore(Arc::new(Mutex::new(HashMap::<String, AuthState>::new())));

    let cached_oidc_provider_metadata = Cached::<CoreProviderMetadata, anyhow::Error>::new(
        oidc_provider_metadata,
        std::time::Duration::from_secs(60),
    )
    .await?;

    let cached_oidc_client =
        Cached::<CoreClient, anyhow::Error>::new(oidc_client, std::time::Duration::from_secs(60))
            .await?;

    let app = Router::new()
        .route(
            "/api/graphql",
            get(graphiql).post_service(GraphQL::new(schema)),
        )
        .route("/api/auth/login", get(auth_login))
        .route("/api/auth/redirect", get(auth_redirect))
        .route("/internal/auth_request", get(auth::auth_request))
        .layer(CookieManagerLayer::new())
        .layer(Extension(auth_state_store))
        .layer(Extension(cached_oidc_provider_metadata))
        .layer(Extension(cached_oidc_client));

    Ok(axum::serve(TcpListener::bind("0.0.0.0:8000").await?, app).await?)
}
