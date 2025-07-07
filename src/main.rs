#![allow(non_snake_case)]

mod dlMgr;

use std::fs::File;
use std::io::{stdin, Error, Write};
use std::process::Command;
use std::time::Duration;
use std::{fs, io};
use tokio::io::{stdout, AsyncWriteExt};
use tokio::time::sleep;
use crate::dlMgr::DlMgr;

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    println!("Minecraft Server Manager - V1.0");
    if !fs::exists("server.jar")? {//Srv not dl-ed yet.
        print!("Version to download: ");
        io::stdout().flush().unwrap();
        let mut ver = String::new();
        stdin().read_line(&mut ver)?;
        let lTask =tokio::spawn(loading());
        let dl = DlMgr::init(ver);
        if !dl.fetch().await?.download().await?.verify().await? {
            println!("Hash mismatch");
            return Ok(())
        }
        println!("Hash match");
        
        lTask.abort();
        println!();
        if !accept_eula()? {
            return Ok(())
        }
    }
    if !fs::exists("eula.txt")? {
        if !accept_eula()? {
            return Ok(())
        }
    }
    start_srv();

    Ok(())
}

fn start_srv() {
    if let Err(e) = Command::new("java").arg("-jar").arg("server.jar").arg("--nogui").status() {
        println!("Failed to start the server: {}", e);
    }
}

fn accept_eula()->Result<bool,Error> {
    print!("Do you agree to the eula? (https://aka.ms/MinecraftEULA) [Y/N]: ");
    io::stdout().flush()?;
    let mut resp = String::new();
    stdin().read_line(&mut resp)?;
    match resp.as_str().trim() {
        "Y" | "y" => {
            let mut file = File::create("eula.txt")?;
            file.write_all(b"eula=true")?;
            Ok(true)
        },
        "N" | "n" => {
            println!("You will need to agree to the eula to continue.");
            Ok(false)
        },
        _ => {
            println!("Incorrect answer.");
            accept_eula()
        }
    }
}

async fn loading()-> io::Result<()> {
    let mut out = stdout();
    out.write_all(b"Downloading server").await?;
    out.flush().await?;
    loop {
        out.write_all(b".").await?;
        out.flush().await?;
        sleep(Duration::from_millis(200)).await;
    }
}
