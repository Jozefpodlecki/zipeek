use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageStore {
    root: Arc<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredImage {
    pub id: Uuid,
    pub media_type: String,
    pub prompt: String,
    pub width: u32,
    pub height: u32,
}

impl ImageStore {
    pub async fn new(root: PathBuf) -> Result<Self, std::io::Error> {
        tokio::fs::create_dir_all(&root).await?;
        Ok(Self {
            root: Arc::new(root),
        })
    }

    fn extension(media_type: &str) -> std::io::Result<&'static str> {
        match media_type {
            "image/png" => Ok("png"),
            "image/svg+xml" => Ok("svg"),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "unsupported image media type",
            )),
        }
    }

    fn path(&self, id: Uuid, extension: &str) -> PathBuf {
        self.root.join(format!("{id}.{extension}"))
    }

    pub async fn put(&self, image: &StoredImage, data: &[u8]) -> std::io::Result<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.path(image.id, Self::extension(&image.media_type)?))
            .await?;
        use tokio::io::AsyncWriteExt;
        file.write_all(data).await?;
        file.flush().await?;
        let metadata = serde_json::to_vec_pretty(image).map_err(std::io::Error::other)?;
        tokio::fs::write(self.path(image.id, "json"), metadata).await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> std::io::Result<Option<(StoredImage, Vec<u8>)>> {
        let metadata = match tokio::fs::read(self.path(id, "json")).await {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let image: StoredImage =
            serde_json::from_slice(&metadata).map_err(std::io::Error::other)?;
        let data = tokio::fs::read(self.path(id, Self::extension(&image.media_type)?)).await?;
        Ok(Some((image, data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn roundtrip_image_bytes() {
        let dir = std::env::temp_dir().join(format!("zipeek-image-test-{}", Uuid::new_v4()));
        let store = ImageStore::new(dir.clone()).await.unwrap();
        let image = StoredImage {
            id: Uuid::new_v4(),
            media_type: "image/svg+xml".into(),
            prompt: "apple".into(),
            width: 512,
            height: 512,
        };
        store.put(&image, b"<svg/>").await.unwrap();
        assert_eq!(store.get(image.id).await.unwrap().unwrap().1, b"<svg/>");
        assert!(store.get(Uuid::new_v4()).await.unwrap().is_none());
        let png = StoredImage {
            id: Uuid::new_v4(),
            media_type: "image/png".into(),
            prompt: "apple".into(),
            width: 512,
            height: 512,
        };
        store.put(&png, b"png-bytes").await.unwrap();
        assert_eq!(store.get(png.id).await.unwrap().unwrap().1, b"png-bytes");
        tokio::fs::remove_dir_all(dir).await.unwrap();
    }
}
