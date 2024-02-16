use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    extract::{Query, Request},
    response::{self, IntoResponse, Redirect},
    routing::get,
    Extension, Router,
};
use nimble_graphql::QueryRoot;
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreResponseType, CoreUserInfoClaims,
};
use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};
use reqwest::Certificate;
use tokio::net::TcpListener;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}

async fn async_http_client_with_certificate(
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
    use oauth2::reqwest::Error;

    let client = reqwest::Client::builder()
        .add_root_certificate(
            Certificate::from_pem(
                r#"-----BEGIN CERTIFICATE-----
MIIBoDCCAUWgAwIBAgIQb1E6+8caj/BEn9ws+BKwsTAKBggqhkjOPQQDAjAuMREw
DwYDVQQKEwhTb3V0aFBLSTEZMBcGA1UEAxMQU291dGhQS0kgUm9vdCBDQTAeFw0y
NDAyMTUwMjA1MDdaFw0zNDAyMTIwMjA1MDdaMC4xETAPBgNVBAoTCFNvdXRoUEtJ
MRkwFwYDVQQDExBTb3V0aFBLSSBSb290IENBMFkwEwYHKoZIzj0CAQYIKoZIzj0D
AQcDQgAE7oqegcoEJiBm2+Dduf04Pas0e+2ZjrUw6U5IeTITOrCv1n2R110cqW3q
KCMo7v1A/zFqrei99/+jSQX9dNSCLaNFMEMwDgYDVR0PAQH/BAQDAgEGMBIGA1Ud
EwEB/wQIMAYBAf8CAQEwHQYDVR0OBBYEFHAypiVNr+S5RyyEyi96P94/Q+FKMAoG
CCqGSM49BAMCA0kAMEYCIQD8g75k/qSQLerNyetBkeucnrPf5NLj93uyj1Y0EWuF
KQIhAP3b2AvHAX9AQP7FRzUPqsPD/JcUouMvjoDoD4DuI7DZ
-----END CERTIFICATE-----"#
                    .as_bytes(),
            )
            .unwrap(),
        )
        .build()
        .unwrap();

    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::Reqwest)?;

    let response = client.execute(request).await.map_err(Error::Reqwest)?;

    let status_code = response.status();
    let headers = response.headers().to_owned();
    let chunks = response.bytes().await.map_err(Error::Reqwest)?;
    Ok(openidconnect::HttpResponse {
        status_code,
        headers,
        body: chunks.to_vec(),
    })
}

struct AuthState {
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
}

#[derive(Clone)]
pub struct AuthStateStore(Arc<Mutex<HashMap<String, AuthState>>>);

async fn auth_login(
    Extension(AuthStateStore(auth_state_store)): Extension<AuthStateStore>,
) -> impl IntoResponse {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://auth.yuru.site/realms/kmc".to_string()).unwrap(),
        async_http_client_with_certificate,
    )
    .await
    .unwrap();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("nimble".to_string()),
        None,
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:3000/api/auth/redirect".to_string()).unwrap(),
    );

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let mut auth_state_store = auth_state_store.lock().unwrap();
    auth_state_store.insert(
        csrf_token.secret().to_string(),
        AuthState {
            nonce,
            pkce_verifier,
        },
    );

    Redirect::temporary(auth_url.as_str())
}

#[derive(serde::Deserialize, Debug)]
struct AuthRedirectQuery {
    code: String,
    iss: String,
    session_state: String,
    state: String,
}

async fn auth_redirect(
    Query(query): Query<AuthRedirectQuery>,
    Extension(AuthStateStore(auth_state_store)): Extension<AuthStateStore>,
) -> impl IntoResponse {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://auth.yuru.site/realms/kmc".to_string()).unwrap(),
        async_http_client_with_certificate,
    )
    .await
    .unwrap();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("nimble".to_string()),
        None,
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:3000/api/auth/redirect".to_string()).unwrap(),
    );

    let auth_state = {
        let mut auth_state_store = auth_state_store.lock().unwrap();
        auth_state_store.remove(&query.state).unwrap()
    };

    let token_response = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(auth_state.pkce_verifier)
        .request_async(async_http_client_with_certificate)
        .await
        .unwrap();

    let id_token = token_response.id_token().unwrap();
    let claims = id_token
        .claims(&client.id_token_verifier(), &auth_state.nonce)
        .unwrap();

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg().unwrap(),
        )
        .unwrap();
        if actual_access_token_hash != *expected_access_token_hash {
            // return Err(anyhow::anyhow!("Invalid access token"));
            panic!("Invalid access token hash");
        }
    }

    let response = format!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>"),
    );
    // println!("{}", response);
    response
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let auth_state_store =
        AuthStateStore(Arc::new(Mutex::new(HashMap::<String, AuthState>::new())));

    let app = Router::new()
        .route(
            "/api/graphql",
            get(graphiql).post_service(GraphQL::new(schema)),
        )
        .route("/api/auth/login", get(auth_login))
        .route("/api/auth/redirect", get(auth_redirect))
        .layer(Extension(auth_state_store));

    axum::serve(TcpListener::bind("0.0.0.0:8000").await.unwrap(), app)
        .await
        .unwrap();
}
