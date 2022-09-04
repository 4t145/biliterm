
use std::sync::Arc;
use bilibili_client::{Client, ClientConfig};

pub struct WebApiService {
    pub bilibili: Arc<Client>,
}

use crate::error::Error;
impl WebApiService {
    pub fn new() -> Result<Self, Error> {
        use std::path::Path;
        let config = ClientConfig {
            cookie_file: Some(Path::new("./webapi.cookie")),
        };
        let client = Client::new(config).map_err(Error::WebApiClientFail)?;


        Ok(Self {
            bilibili: client,
        })
    }
}