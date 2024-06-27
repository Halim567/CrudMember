#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claims {
    email: String,
    exp: usize,
}

impl Claims {
    pub fn new(email: String) -> Self {
        let duration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(60))
            .expect("Invalid timestamp")
            .timestamp() as usize;

        Self {
            email,
            exp: duration,
        }
    }

    pub fn to_token(&self) -> Result<String, axum::http::StatusCode> {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(b"SECRET"),
        )
        .map_err(|err| {
            tracing::error!("{err}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
    }
}
