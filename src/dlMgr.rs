use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::error;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::task::{spawn_blocking, JoinHandle};
//TODO Warn if unsupported, maybe handle java?, start with recommended flags

pub struct DlMgr {
    dlUrl:String,
    sha:String,
    ver:String
}

impl DlMgr {

    pub fn init(ver:String)-> DlMgr {
        DlMgr {dlUrl:String::new(),sha:String::new(),ver}
    }

    pub async fn fetch(&mut self)->Result<&Self, Box<dyn error::Error>> {
        let resp = reqwest::get(format!(
            "https://fill.papermc.io/v3/projects/paper/versions/{}/builds/latest",
            self.ver)).await?;
        let jResp:Value = resp.json().await?;
        if let Some(err) = &jResp["message"].as_str() {
            return Err(Box::new(Error::new(ErrorKind::Other, *err)))
        }
        let rand = &jResp["downloads"]["server:default"];
        self.dlUrl=rand["url"].as_str().unwrap().to_string();
        self.sha=rand["checksums"]["sha256"].as_str().unwrap().to_string();
        Ok(self)
    }
    pub async fn download(&self) -> Result<&Self, Box<dyn error::Error+Send+Sync>> {
        let paper = &self.dlUrl;
        let mut dl_file = reqwest::get(paper).await?.error_for_status()?;
        let mut disk = AsyncFile::create("server.jar").await?;
        while let Some(elem)= dl_file.chunk().await? {
            disk.write_all(&elem).await?;
        }
        Ok(self)
    }
//Last-call, remove self.
    pub async fn verify(self)->Result<bool,Error> {
        let hndl:JoinHandle<Result<bool,Error>> =spawn_blocking(move || {
            let mut hasher = Sha256::new();
            let mut buf = [0u8; 4096];
            let mut srvFile = File::open("server.jar")?;
            loop {
                let br = srvFile.read(&mut buf)?;
                if br<1 {
                    break;
                }
                hasher.update(&buf[..br]);
            }
            let res = hasher.finalize();
            Ok(hex::encode(res)==self.sha)
        });
        hndl.await?
    }
}