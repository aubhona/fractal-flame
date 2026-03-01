use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

#[derive(Clone)]
pub struct MinioClient {
    bucket: std::sync::Arc<Bucket>,
}

#[derive(Clone, Debug)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

impl MinioClient {
    pub fn new(config: MinioConfig) -> Result<Self, MinioError> {
        let region = Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };

        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| MinioError::Creds(e.to_string()))?;

        let mut bucket = Bucket::new(&config.bucket, region, credentials)
            .map_err(|e| MinioError::S3(e.to_string()))?
            .with_request_timeout(std::time::Duration::from_secs(30))
            .map_err(|e| MinioError::S3(e.to_string()))?;
        bucket.set_path_style();

        Ok(Self {
            bucket: std::sync::Arc::new(*bucket),
        })
    }

    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), MinioError> {
        let response = self
            .bucket
            .put_object_with_content_type(key, &body, content_type)
            .await
            .map_err(|e| MinioError::S3(e.to_string()))?;
        if response.status_code() != 200 {
            return Err(MinioError::S3(format!(
                "PUT returned status {}",
                response.status_code()
            )));
        }
        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, MinioError> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| MinioError::S3(e.to_string()))?;
        Ok(response.bytes().to_vec())
    }

    pub async fn ping(&self) -> Result<(), MinioError> {
        self.bucket
            .list("__health__".to_string(), Some("/".to_string()))
            .await
            .map_err(|e| MinioError::S3(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MinioError {
    #[error("S3 error: {0}")]
    S3(String),
    #[error("Credentials error: {0}")]
    Creds(String),
}
