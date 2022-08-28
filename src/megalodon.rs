use crate::entities;
use crate::error::Error;
use crate::response::Response;
use async_trait::async_trait;

#[async_trait]
pub trait Megalodon {
    // async fn verify_account_credentials() -> Account;
    async fn get_instance(&self) -> Result<Response<entities::Instance>, Error>;
}
