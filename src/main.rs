#![allow(non_snake_case)]

mod dlMgr;

use crate::dlMgr::DlMgr;
use std::fs::File;
use std::io::{stdin, Error, Write};
use std::process::{exit, Command};
use std::time::Duration;
use std::{fs, io};
use tokio::io::{stdout, AsyncWriteExt};
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    println!("Minecraft Server Manager - V1.0");
    if !fs::exists("server.jar")? {//Srv not dl-ed yet.
        start_dl().await?;
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

async fn start_dl() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    let mut lTask:JoinHandle<Result<(),Error>>;
    let mut dl:DlMgr;
    //Req user input until it provides a good one.
    loop {
        print!("Version to download: ");
        io::stdout().flush().unwrap();

        let mut ver = String::new();
        stdin().read_line(&mut ver)?;
        stdout().write_all(b"Getting version information...\n").await?;
        dl = DlMgr::init(ver);
        if let Err(e)=dl.fetch().await {
            println!("Error while requesting that version: {}",e);
        } else {
            break;
        }
    }

    lTask=tokio::spawn(loading());
    dl.download().await?;
    lTask.abort();
    stdout().write_all(b"\n").await?;

    lTask=tokio::spawn(hashLoading());
    let isCorrect = dl.verify().await?;
    lTask.abort();
    println!();

    if !isCorrect {
        fs::remove_file("server.jar")?;
        println!("Download hash mismatch! Press enter to exit...");
        stdin().read_line(&mut String::new())?;
        exit(0)
    }
    println!("Download hash verified.");
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
//TODO File dl progress display
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

async fn hashLoading()-> io::Result<()> {
    let mut out = stdout();
    out.write_all(b"Verifying hash").await?;
    out.flush().await?;
    loop {
        out.write_all(b".").await?;
        out.flush().await?;
        sleep(Duration::from_millis(200)).await;
    }
}
