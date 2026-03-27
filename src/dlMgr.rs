use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use ureq::Agent;
//TODO Warn if unsupported, maybe handle java?

pub struct DlMgr {
    dlUrl:String,
    sha:[u8; 64],
    ver:String,
    build:u64,//Build
    isPaper:bool,
    client:Agent
}

impl DlMgr {
    pub fn init(ver:String, isPaper:bool)-> DlMgr {
        DlMgr {dlUrl:String::new(),sha:[0u8;64],ver,build:0,isPaper,client:getAgent()}
    }

    pub fn fetch(&mut self)->Result<&Self, Box<dyn Error>> {
        if self.ver.is_empty() {
            self.ver=self.getLatVer()?;
            println!("The latest version: {}",self.ver);
        }
        let mut resp = self.client.get(format!("https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest", if self.isPaper {"paper"} else {"velocity"}, self.ver)).call()?;
        let jResp:Value = resp.body_mut().read_json()?;
        if let Some(err) = jResp["message"].as_str() {
            return Err(err.into());
        }
        self.build = jResp["id"].as_u64().unwrap();
        let latestVer = &jResp["downloads"]["server:default"];
        self.dlUrl = latestVer["url"].as_str().unwrap().to_string();
        self.sha = <[u8; 64]>::try_from(latestVer["checksums"]["sha256"].as_str().unwrap().as_bytes())?;
        //self.sha[4] = b'c';  //To test the hash verification
        Ok(self)
    }//TODO REUSE Agent after upd check?
    pub fn downloadAndVerify(&self) -> Result<bool, Box<dyn Error>> {
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
//Decode the hex to a string so it can be compared
        let mut hex_bytes = [0u8; 64];
        let chars = b"0123456789abcdef";
        for (i, &b) in res.iter().enumerate() {
            hex_bytes[i * 2] = chars[(b >> 4) as usize];
            hex_bytes[i * 2 + 1] = chars[(b & 0xf) as usize];
        }

        pb.finish();
        Ok(hex_bytes==self.sha)
    }

    fn getLatVer(&self) -> Result<String, ureq::Error> {
        let json:Value = self.client.get(format!("https://fill.papermc.io/v3/projects/{}/versions",if self.isPaper {"paper"} else {"velocity"})).call()?.body_mut().read_json()?;
        let ver = json["versions"][0]["version"]["id"].as_str().unwrap();//Get the first ver
        Ok(ver.to_string())
    }
//Util
    fn decName(&self)->String {
        format!("server{}{}_{}.jar", if self.isPaper { "_" } else { "V_" }, self.ver, self.build)
    }
}

pub fn getLatBuild(ver:&mut String, isPaper:bool) -> Result<u64,Box<dyn Error>> {
    let ans:Value = getAgent().get(format!(
        "https://fill.papermc.io/v3/projects/{}/versions/{}/builds/latest",if isPaper {"paper"} else {"velocity"},ver)).call()?.body_mut().read_json()?;
    ans["id"].as_u64().ok_or("Failed to get latest build ID".into())
}

fn getAgent() -> Agent {
    Agent::config_builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), " (https://github.com/GRX005/McSrvMgr)"))
        .https_only(true)
        .http_status_as_error(false)
        .build()
        .new_agent()
}
