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
    ver:String,
    build:u64,//Build
    isPaper:bool
}

impl DlMgr {

    pub fn init(ver:String, isPaper:bool)-> DlMgr {
        DlMgr {dlUrl:String::new(),sha:String::new(),ver:ver.trim().to_string(),build:0,isPaper}
    }

    pub async fn fetch(&mut self)->Result<&Self, Box<dyn error::Error>> {
        if self.ver.trim().is_empty() {
            self.ver=self.getLatest().await?;
            println!("The latest version: {}",self.ver);
        }
        let resp = reqwest::get(format!("https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest", if self.isPaper {"paper"} else {"velocity"},self.ver)).await?;
        if self.ver.contains("-SNAPSHOT") {
            self.ver=self.ver.replace("-SNAPSHOT", "");
        }
        let jResp:Value = resp.json().await?;
        if let Some(err) = jResp["message"].as_str() {
            return Err(Box::new(Error::new(ErrorKind::Other, err)))
        }
        self.build =jResp["id"].as_u64().unwrap();
        let rand = &jResp["downloads"]["server:default"];
        self.dlUrl=rand["url"].as_str().unwrap().to_string();
        self.sha=rand["checksums"]["sha256"].as_str().unwrap().to_string();
        //self.sha=self.sha.replace("9","j");
        Ok(self)
    }
    pub async fn download(&self) -> Result<&Self, Box<dyn error::Error+Send+Sync>> {
        let paper = &self.dlUrl;
        let mut dl_file = reqwest::get(paper).await?.error_for_status()?;
        let mut disk = AsyncFile::create(self.decType()).await?;
        while let Some(elem)= dl_file.chunk().await? {
            disk.write_all(&elem).await?;
        }
        Ok(self)
    }
//An AI-gen func that gets latest ver automatically, should be the simplest+most optimal+supports 2.x+ ver scheme too.
    async fn getLatest(&self) -> Result<String, Box<dyn error::Error>> {
        let json: Value = reqwest::get(format!("https://fill.papermc.io/v3/projects/{}",if self.isPaper {"paper"} else {"velocity"})).await?.json().await?;

        let ver = json["versions"].as_object().unwrap();

        let latest = ver.keys().max_by_key(|k| {
            let parts: Vec<u32> = k.split('.').map(|s| s.parse().unwrap()).collect();
            (parts[0], parts[1])
        }).unwrap();

        Ok(ver[latest].as_array().unwrap()[0].as_str().unwrap().to_string())
    }
//Last-call, remove self.
    pub async fn verify(self)->Result<bool,Error> {
        let hndl:JoinHandle<Result<bool,Error>> =spawn_blocking(move || {
            let mut hasher = Sha256::new();
            let mut buf = [0u8; 4096];
            let mut srvFile = File::open(self.decType())?;
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
//Util
    fn decType(&self)->String {
        "server".to_owned()+if self.isPaper {"-"} else {"V-"}+ &self.ver+"-"+&self.build.to_string()+".jar"
    }
}

pub async fn getLatBuild(ver:&mut String, isPaper:bool) -> u64 {
    if !isPaper {
        ver.push_str("-SNAPSHOT")
    }
    let ans:Value = reqwest::get(format!(
        "https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest",if isPaper {"paper"} else {"velocity"},ver)).await.unwrap().json().await.unwrap();
    ans["id"].as_u64().unwrap()
}
