use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};
use std::{future::Future, pin::Pin};

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
    Extension,
};

use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreJsonWebKeyType,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, CsrfToken, EmptyAdditionalClaims, IdToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};

use reqwest::Certificate;
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};

use crate::utilities::{
    axum::{bad_request, internal_server_error},
    Cached,
};

/// This is the openidconnect::reqwest::async_http_client function with a certificate added.
pub async fn async_http_client_with_certificate(
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

pub fn oidc_provider_metadata(
) -> Pin<Box<dyn Future<Output = anyhow::Result<CoreProviderMetadata>> + Send>> {
    Box::pin(async {
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new("https://auth.yuru.site/realms/kmc".to_string()).unwrap(),
            async_http_client_with_certificate,
        )
        .await
        .unwrap();

        Ok(provider_metadata)
    })
}

pub fn oidc_client() -> Pin<Box<dyn Future<Output = anyhow::Result<CoreClient>> + Send>> {
    Box::pin(async {
        let provider_metadata = oidc_provider_metadata().await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new("nimble".to_string()),
            None,
        )
        .set_redirect_uri(RedirectUrl::new(
            "http://localhost:3000/api/auth/redirect".to_string(),
        )?);

        Ok(client)
    })
}

pub struct AuthState {
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
}

#[derive(Clone)]
pub struct AuthStateStore(pub Arc<Mutex<HashMap<String, AuthState>>>);

pub async fn auth_login(
    Extension(AuthStateStore(auth_state_store)): Extension<AuthStateStore>,
    Extension(cached_oidc_client): Extension<Cached<CoreClient, anyhow::Error>>,
) -> Result<Response, Response> {
    let client = cached_oidc_client
        .get()
        .await
        .map_err(|_| internal_server_error("Failed to get OIDC client"))?;

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

    let mut auth_state_store = auth_state_store.lock().await;
    auth_state_store.insert(
        csrf_token.secret().to_string(),
        AuthState {
            nonce,
            pkce_verifier,
        },
    );

    Ok(Redirect::temporary(auth_url.as_str()).into_response())
}

#[derive(serde::Deserialize, Debug)]
pub struct AuthRedirectQuery {
    code: String,
    #[allow(unused)]
    iss: String,
    #[allow(unused)]
    session_state: String,
    state: String,
}

pub async fn auth_redirect(
    Query(query): Query<AuthRedirectQuery>,
    Extension(AuthStateStore(auth_state_store)): Extension<AuthStateStore>,
    Extension(cached_oidc_client): Extension<Cached<CoreClient, anyhow::Error>>,
    cookies: Cookies,
) -> Result<Response, Response> {
    let auth_state = {
        let mut auth_state_store = auth_state_store.lock().await;
        auth_state_store
            .remove(&query.state)
            .ok_or_else(|| bad_request("Invalid state"))?
    };

    let client = cached_oidc_client
        .get()
        .await
        .map_err(|_| internal_server_error("Failed to get OIDC client"))?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(auth_state.pkce_verifier)
        .request_async(async_http_client_with_certificate)
        .await
        .map_err(|_| internal_server_error("Failed to exchange code for token"))?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| bad_request("Missing ID token"))?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &auth_state.nonce)
        .map_err(|_| bad_request("Invalid ID token"))?;

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token
                .signing_alg()
                .map_err(|_| bad_request("Invalid signing algorithm"))?,
        )
        .map_err(|_| bad_request("Failed to calculate access token hash"))?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(bad_request("Invalid access token hash"));
        }
    }

    // let response = format!(
    //     "User {} with e-mail address {} has authenticated successfully",
    //     claims
    //         .preferred_username()
    //         .map(|username| username.as_str())
    //         .unwrap_or("<not provided>"),
    //     claims
    //         .email()
    //         .map(|email| email.as_str())
    //         .unwrap_or("<not provided>"),
    // );

    // cookies.add(
    //     Cookie::build((
    //         "access_token",
    //         token_response.access_token().secret().to_string(),
    //     ))
    //     .http_only(true)
    //     .secure(false)
    //     .path("/")
    //     .build(),
    // );

    cookies.add(
        Cookie::build((
            "id_token",
            token_response
                .id_token()
                .ok_or_else(|| bad_request("Missing ID token"))?
                .to_string(),
        ))
        .http_only(true)
        .secure(false)
        .path("/")
        .build(),
    );

    Ok((Redirect::temporary("/"),).into_response())
}

pub async fn auth_request(
    cookies: Cookies,
    cached_oidc_client: Extension<Cached<CoreClient, anyhow::Error>>,
) -> Result<Response, Response> {
    if let Some(id_token_cookie) = cookies.get("id_token") {
        let _client = cached_oidc_client
            .get()
            .await
            .map_err(|_| internal_server_error("Failed to get OIDC provider metadata"))?;
        let _id_token = IdToken::<
            EmptyAdditionalClaims,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
            CoreJsonWebKeyType,
        >::from_str(id_token_cookie.value())
        .map_err(|_| internal_server_error("Failed to parse ID token"))?;
        // ここの nonce どうするん？？
        // let claims = id_token.claims(&client.id_token_verifier(), &Nonce::new("".to_string()))?;
        // TODO: make sure whether to use access token or id token
        // TODO: verify id token or access token correctly
        Ok(Response::builder()
            .status(200)
            .body("Authenticated".into())
            .unwrap())
    } else {
        Err(Response::builder()
            .status(500)
            .body("Not authenticated".into())
            .unwrap())
    }
}
