use crate::DlMsg;
use reqwest::{blocking, tls, ClientBuilder};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::error;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
//TODO Warn if unsupported, maybe handle java?

pub struct DlMgr {
    dlUrl:String,
    sha:String,
    ver:String,
    build:u64,//Build
    isPaper:bool,
    client:blocking::Client
}

macro_rules! getClient {
    ($builder:expr) => {
        $builder
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"), "/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/GRX005/McSrvMgr)"
            ))
            .min_tls_version(tls::Version::TLS_1_3)
            .https_only(true)
            .tls_backend_rustls()
            .build()
            .unwrap()
    };
}

impl DlMgr {

    pub fn init(ver:String, isPaper:bool)-> DlMgr {
        DlMgr {dlUrl:String::new(),sha:String::new(),ver:ver.trim().to_string(),build:0,isPaper,client:getClient!(blocking::ClientBuilder::new())}
    }

    pub fn fetch(&mut self)->Result<&Self, Box<dyn error::Error>> {
        if self.ver.trim().is_empty() {
            self.ver=self.getLatest()?;
            println!("The latest version: {}",self.ver);
        }
        let resp = self.client.get(format!("https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest", if self.isPaper {"paper"} else {"velocity"},self.ver)).send()?;
        if self.ver.contains("-SNAPSHOT") {
            self.ver=self.ver.replace("-SNAPSHOT", "");
        }
        let jResp:Value = resp.json()?;
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
    pub async fn download(self, tx:Sender<DlMsg>) -> Result<Self, Box<dyn error::Error+Send+Sync>> {
        let asyncClient = getClient!(ClientBuilder::new());
        let mut dl_file = asyncClient.get(&self.dlUrl).send().await?.error_for_status()?;
        let totalSize = dl_file.content_length().unwrap_or(0);
        tx.send(DlMsg::StartWithSize(totalSize)).await?;
        //let size = dl_file.content_length().unwrap();
        let mut disk = AsyncFile::create(self.decName()).await?;
        while let Some(elem)= dl_file.chunk().await? {
            disk.write_all(&elem).await?;
            tx.send(DlMsg::Chunk(elem.len() as u64)).await?
        }
        Ok(self)
    }
//An AI-gen func that gets latest ver automatically, should be the simplest+most optimal+supports 2.x+ ver scheme too.
    fn getLatest(&self) -> Result<String, Box<dyn error::Error>> {
        let json: Value = self.client.get(format!("https://fill.papermc.io/v3/projects/{}",if self.isPaper {"paper"} else {"velocity"})).send()?.json()?;

        let ver = json["versions"].as_object().unwrap();

        let latest = ver.keys().max_by_key(|k| {
            let parts: Vec<u32> = k.split('.').map(|s| s.parse().unwrap()).collect();
            (parts[0], parts[1])
        }).unwrap();

        Ok(ver[latest].as_array().unwrap()[0].as_str().unwrap().to_string())
    }
//Last-call, remove self.
    pub fn verify(self)->Result<bool,Error> {
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 4096];
        let mut srvFile = File::open(self.decName())?;
        loop {
            let br = srvFile.read(&mut buf)?;
            if br<1 {
                break;
            }
            hasher.update(&buf[..br]);
        }
        let res = hasher.finalize();
        Ok(hex::encode(res)==self.sha)
    }
//Util
    fn decName(&self)->String {
        "server".to_owned()+if self.isPaper {"-"} else {"V-"}+ &self.ver+"-"+&self.build.to_string()+".jar"
    }
}

pub fn getLatBuild(ver:&mut String, isPaper:bool) -> Result<u64,Box<dyn error::Error>> {
    if !isPaper {
        ver.push_str("-SNAPSHOT")
    }
    let ans:Value = getClient!(blocking::ClientBuilder::new()).get(format!(
        "https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest",if isPaper {"paper"} else {"velocity"},ver)).send()?.json()?;
    ans["id"].as_u64().ok_or("Failed to get latest build ID".into())
}
