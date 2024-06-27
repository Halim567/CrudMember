use axum::http::{header::AUTHORIZATION, request::Parts};
use jwt::Claims;

pub mod controllers;
pub mod jwt;

pub struct QueryHeader(String);

impl QueryHeader {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[axum::async_trait]
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for QueryHeader {
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Tidak diizinkan"))?
            .to_str()
            .map_err(|err| {
                tracing::error!("{err}");
                (axum::http::StatusCode::BAD_REQUEST, "Header tidak valid")
            })?;

        if !header_value.starts_with("Bearer ") {
            return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid Header Value"));
        }

        let token = header_value.trim_start_matches("Bearer ");
        jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(b"SECRET"),
            &jsonwebtoken::Validation::default(),
        )
        .map(|_| Self("Ok".to_owned()))
        .map_err(|err| {
            tracing::error!("{err}");
            (axum::http::StatusCode::UNAUTHORIZED, "Invalid Token")
        })
    }
}

#[derive(Debug)]
pub struct InvalidEmail;

impl std::fmt::Display for InvalidEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email is invalid")
    }
}

impl std::error::Error for InvalidEmail {}

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(raw_email: String) -> Result<Self, InvalidEmail> {
        let rgx = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if !rgx.is_match(&raw_email) {
            return Err(InvalidEmail);
        }

        Ok(Email(raw_email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Email {
    fn default() -> Self {
        Self(String::from("user@example.com"))
    }
}

impl<'de> serde::Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_email = String::deserialize(deserializer)?;
        Self::new(raw_email).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
