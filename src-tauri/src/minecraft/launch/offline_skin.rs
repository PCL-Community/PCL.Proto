use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::rngs::OsRng;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1v15::SigningKey,
    pkcs8::{EncodePublicKey, LineEnding},
    signature::{SignatureEncoding, Signer},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

use crate::minecraft::profile::McProfile;

#[derive(Clone)]
struct SkinServerState {
    uuid: String,
    name: String,
    metadata: Value,
    profile: Value,
    image: Vec<u8>,
}

pub struct SkinServer {
    task: tokio::task::JoinHandle<()>,
    pub url: String,
    pub metadata: String,
}

impl Drop for SkinServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn start(profile: &McProfile) -> Result<SkinServer, String> {
    let skin = profile
        .offline_skin
        .as_ref()
        .ok_or("Offline skin is missing")?;
    let image = STANDARD
        .decode(&skin.image)
        .map_err(|error| error.to_string())?;
    let uuid = uuid::Uuid::parse_str(&profile.uuid)
        .map_err(|error| format!("Invalid offline UUID: {error}"))?
        .simple()
        .to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let url = format!(
        "http://127.0.0.1:{}",
        listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port()
    );
    let key = tokio::task::spawn_blocking(|| RsaPrivateKey::new(&mut OsRng, 2048))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let public_key = RsaPublicKey::from(&key)
        .to_public_key_pem(LineEnding::LF)
        .map_err(|error| error.to_string())?;
    let metadata = json!({
        "signaturePublickey": public_key,
        "skinDomains": ["127.0.0.1", "localhost"],
        "meta": {"serverName": "PCL.Proto", "implementationName": "PCL.Proto", "implementationVersion": "1.0", "feature.non_email_login": true}
    });
    let mut skin_texture = json!({"url": format!("{url}/textures/skin.png")});
    if skin.slim {
        skin_texture["metadata"] = json!({"model": "slim"});
    }
    let textures = json!({"timestamp": chrono::Utc::now().timestamp_millis(), "profileId": uuid, "profileName": profile.user_name, "textures": {"SKIN": skin_texture}});
    let value = STANDARD.encode(serde_json::to_vec(&textures).map_err(|error| error.to_string())?);
    let signature = STANDARD.encode(
        SigningKey::<sha1_legacy::Sha1>::new(key)
            .sign(value.as_bytes())
            .to_bytes(),
    );
    let response = json!({"id": uuid, "name": profile.user_name, "properties": [{"name": "textures", "value": value, "signature": signature}]});
    let state = Arc::new(SkinServerState {
        uuid,
        name: profile.user_name.clone(),
        metadata: metadata.clone(),
        profile: response,
        image,
    });
    let router = Router::new()
        .route("/", get(root))
        .route("/api/profiles/minecraft", post(profiles))
        .route(
            "/sessionserver/session/minecraft/profile/{uuid}",
            get(profile_by_uuid),
        )
        .route(
            "/sessionserver/session/minecraft/hasJoined",
            get(has_joined),
        )
        .route(
            "/sessionserver/session/minecraft/join",
            post(|| async { StatusCode::NO_CONTENT }),
        )
        .route("/textures/skin.png", get(texture))
        .with_state(state);
    let task = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            log::error!("Offline skin service failed: {error}");
        }
    });
    Ok(SkinServer {
        task,
        url,
        metadata: STANDARD.encode(metadata.to_string()),
    })
}

async fn root(State(state): State<Arc<SkinServerState>>) -> Json<Value> {
    Json(state.metadata.clone())
}

async fn profiles(
    State(state): State<Arc<SkinServerState>>,
    Json(names): Json<Vec<String>>,
) -> Json<Value> {
    Json(
        if names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&state.name))
        {
            json!([{"id": state.uuid, "name": state.name}])
        } else {
            json!([])
        },
    )
}

async fn profile_by_uuid(
    State(state): State<Arc<SkinServerState>>,
    Path(uuid): Path<String>,
) -> Response {
    if uuid.eq_ignore_ascii_case(&state.uuid) {
        Json(state.profile.clone()).into_response()
    } else {
        StatusCode::NO_CONTENT.into_response()
    }
}

async fn has_joined(
    State(state): State<Arc<SkinServerState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if query
        .get("username")
        .is_some_and(|name| name.eq_ignore_ascii_case(&state.name))
    {
        Json(state.profile.clone()).into_response()
    } else {
        StatusCode::NO_CONTENT.into_response()
    }
}

async fn texture(State(state): State<Arc<SkinServerState>>) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/png")], state.image.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::{pkcs8::DecodePublicKey, signature::Verifier};

    #[tokio::test]
    async fn serves_signed_profile_and_only_its_own_skin() {
        let mut account = McProfile::offline("Player", "5627dd98e6be3c21b8a8e92344183641");
        let skin_image = include_bytes!("../../../../public/pcl/images/Skins/Steve.png");
        account.offline_skin = Some(crate::minecraft::profile::OfflineSkin {
            image: STANDARD.encode(skin_image),
            slim: true,
        });
        let server = start(&account).await.unwrap();
        let client = reqwest::Client::new();
        let metadata: Value = client
            .get(&server.url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let public =
            RsaPublicKey::from_public_key_pem(metadata["signaturePublickey"].as_str().unwrap())
                .unwrap();
        let profile: Value = client
            .get(format!(
                "{}/sessionserver/session/minecraft/profile/{}?unsigned=false",
                server.url, account.uuid
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let property = &profile["properties"][0];
        let value = property["value"].as_str().unwrap();
        let signature = STANDARD
            .decode(property["signature"].as_str().unwrap())
            .unwrap();
        let signature = rsa::pkcs1v15::Signature::try_from(signature.as_slice()).unwrap();
        rsa::pkcs1v15::VerifyingKey::<sha1_legacy::Sha1>::new(public)
            .verify(value.as_bytes(), &signature)
            .unwrap();
        let textures: Value = serde_json::from_slice(&STANDARD.decode(value).unwrap()).unwrap();
        assert_eq!(textures["textures"]["SKIN"]["metadata"]["model"], "slim");
        let image = client
            .get(textures["textures"]["SKIN"]["url"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(&image[..], skin_image);
        assert_eq!(
            client
                .get(format!(
                    "{}/sessionserver/session/minecraft/profile/{}",
                    server.url,
                    "0".repeat(32)
                ))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        let url = server.url.clone();
        drop(server);
        tokio::task::yield_now().await;
        assert!(client.get(url).send().await.is_err());
    }
}
