use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::error;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use ureq::Agent;
//TODO Warn if unsupported, maybe handle java?

pub struct DlMgr {
    dlUrl:String,
    sha:String,
    ver:String,
    build:u64,//Build
    isPaper:bool,
    client:Agent
}

impl DlMgr {

    pub fn init(ver:String, isPaper:bool)-> DlMgr {
        DlMgr {dlUrl:String::new(),sha:String::new(),ver,build:0,isPaper,client: getAgent()}
    }

    pub fn fetch(&mut self)->Result<&Self, Box<dyn error::Error>> {
        if self.ver.trim().is_empty() {
            self.ver=self.getLatest()?;
            println!("The latest version: {}",self.ver);
        }
        let mut resp = self.client.get(format!("https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest", if self.isPaper {"paper"} else {"velocity"}, self.ver)).call()?;
        if self.ver.contains("-SNAPSHOT") {
            self.ver=self.ver.replace("-SNAPSHOT", "");
        }
        let jResp:Value = resp.body_mut().read_json()?;
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
    pub fn download(self) -> Result<bool, Box<dyn error::Error+Send+Sync>> {
        let dl_file = self.client.get(&self.dlUrl).call()?;
        let size = dl_file.body().content_length().unwrap_or(0);
        let mut hasher = Sha256::new();

        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::with_template(
                "{msg} {percent:>3}% [{bar:50}] {bytes:>10}/{total_bytes:<10}"
            )?.progress_chars("━╸ "),
        );
        pb.set_message("Downloading and verifying server...");
//TODO Timeout?
        let mut disk = File::create(self.decName())?;
        let mut buf = [0u8; 64 * 1024];
        let mut reader = dl_file.into_body().into_reader();
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 { break; }
            disk.write_all(&buf[..n])?;
            hasher.update(&buf[..n]);
            pb.inc(n as u64);
        }

        let res = hasher.finalize();
        pb.finish();
        Ok(format!("{:x}", res)==self.sha)
    }
//An AI-gen func that gets latest ver automatically, should be the simplest+most optimal+supports 2.x+ ver scheme too.
    fn getLatest(&self) -> Result<String, Box<dyn error::Error>> {
        let json:Value = self.client.get(format!("https://fill.papermc.io/v3/projects/{}",if self.isPaper {"paper"} else {"velocity"})).call()?.body_mut().read_json()?;

        let ver = json["versions"].as_object().unwrap();

        let latest = ver.keys().max_by_key(|k| {
            let parts: Vec<u32> = k.split('.').map(|s| s.parse().unwrap()).collect();
            (parts[0], parts[1])
        }).unwrap();

        Ok(ver[latest].as_array().unwrap()[0].as_str().unwrap().to_string())
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
    let ans:Value = getAgent().get(format!(
        "https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest",if isPaper {"paper"} else {"velocity"},ver)).call()?.body_mut().read_json()?;
    ans["id"].as_u64().ok_or("Failed to get latest build ID".into())
}

fn getAgent() -> Agent {
    Agent::config_builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), " (https://github.com/GRX005/McSrvMgr)"))
        .https_only(true)
        .build()
        .new_agent()
}
