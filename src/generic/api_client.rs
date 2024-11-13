use crate::default::DEFAULT_UA;
use crate::error::{Error as MegalodonError, Kind};
use crate::response::Response;
use reqwest::header::HeaderMap;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct APIClient {
    access_token: Option<String>,
    base_url: String,
    client: reqwest::Client,
}

enum Params<'a> {
    Form(reqwest::multipart::Form),
    Json(&'a HashMap<&'a str, Value>),
}

impl APIClient {
    pub fn new(base_url: String, access_token: Option<String>, user_agent: Option<String>) -> Result<Self, MegalodonError> {
        let ua: String;
        match user_agent {
            Some(agent) => ua = agent,
            None => ua = DEFAULT_UA.to_string(),
        }

        let client = reqwest::Client::builder()
            .user_agent(ua)
            .build()?;

        Ok(Self {
            access_token,
            base_url,
            client,
        })
    }

    async fn request<T>(
        &self,
        path: &str,
        headers: Option<HeaderMap>,
        method: reqwest::Method,
        params: Option<Params<'_>>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        let url_str = format!("{}{}", self.base_url, path);
        let url = Url::parse(&*url_str)?;

        let mut req = self.client.request(method, url);
        if let Some(token) = &self.access_token {
            req = req.bearer_auth(token);
        }
        if let Some(headers) = headers {
            req = req.headers(headers);
        }

        let res = match params {
            None => req,
            Some(Params::Form(form)) => req.multipart(form),
            Some(Params::Json(json)) => req.json(&json),
        }
        .send()
        .await?;

        let res_headers = res.headers().clone();
        let status = res.status();

        match status {
            reqwest::StatusCode::OK
            | reqwest::StatusCode::CREATED
            | reqwest::StatusCode::ACCEPTED
            | reqwest::StatusCode::NO_CONTENT => {
                let res = Response::<T>::from_reqwest(res).await?;
                Ok(res)
            }
            reqwest::StatusCode::PARTIAL_CONTENT => Err(MegalodonError::new_own(
                String::from("The requested resource is still being processed"),
                Kind::HTTPPartialContentError,
                Some(url_str),
                Some(status.as_u16()),
                Some(res_headers),
            )),
            _ => match res.text().await {
                Ok(text) => Err(MegalodonError::new_own(
                    text,
                    Kind::HTTPStatusError,
                    Some(url_str),
                    Some(status.as_u16()),
                    Some(res_headers),
                )),
                Err(_err) => Err(MegalodonError::new_own(
                    "Unknown error".to_string(),
                    Kind::HTTPStatusError,
                    Some(url_str),
                    Some(status.as_u16()),
                    Some(res_headers),
                )),
            },
        }
    }

    pub async fn get<T>(
        &self,
        path: &str,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(path, headers, reqwest::Method::GET, None)
            .await
    }

    pub async fn post<T>(
        &self,
        path: &str,
        params: &HashMap<&str, Value>,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::POST,
            Some(Params::Json(params)),
        )
        .await
    }

    pub async fn post_multipart<T>(
        &self,
        path: &str,
        params: reqwest::multipart::Form,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::POST,
            Some(Params::Form(params)),
        )
        .await
    }

    pub async fn put<T>(
        &self,
        path: &str,
        params: &HashMap<&str, Value>,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::PUT,
            Some(Params::Json(params)),
        )
        .await
    }

    pub async fn put_multipart<T>(
        &self,
        path: &str,
        params: reqwest::multipart::Form,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::PUT,
            Some(Params::Form(params)),
        )
        .await
    }

    pub async fn patch<T>(
        &self,
        path: &str,
        params: &HashMap<&str, Value>,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::PATCH,
            Some(Params::Json(params)),
        )
        .await
    }

    pub async fn delete<T>(
        &self,
        path: &str,
        params: &HashMap<&str, Value>,
        headers: Option<HeaderMap>,
    ) -> Result<Response<T>, MegalodonError>
    where
        T: DeserializeOwned + Debug,
    {
        self.request(
            path,
            headers,
            reqwest::Method::DELETE,
            Some(Params::Json(params)),
        )
        .await
    }
}
