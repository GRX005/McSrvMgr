#![allow(non_snake_case)]

mod dlMgr;

use tokio::io::{stdin, AsyncBufReadExt, AsyncReadExt, BufReader};
use crate::dlMgr::DlMgr;
use std::fs::File;
use std::io::{Error, Write};
use std::process::{exit, Command};
use std::time::Duration;
use std::{error, fs, io};
use tokio::io::{stdout, AsyncWriteExt};
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn error::Error+Send+Sync>> {
    stdout().write_all(b"Minecraft Server Manager - V1.0\n").await?;
    let mut srv = checkLat().await;
    if srv.is_none() {
        start_dl(None).await?;
        srv = checkLat().await;
    }
    //checkLat(srv.as_ref().unwrap()).await;
    if !fs::exists("eula.txt")? {
        if !accept_eula()? {
            return Ok(())
        }
    }
    start_srv(srv.unwrap());

    Ok(())
}

fn getSrvName() -> Option<String> {
    fs::read_dir(".").unwrap().find_map(|v| {
        v.unwrap().file_name().to_str().filter(|s| s.starts_with("server")).filter(|s| s.ends_with(".jar")).map(str::to_owned)
    })
}

async fn checkLat() -> Option<String> {
    let name = getSrvName();
    if name.is_none() {
        return None;
    }
    stdout().write_all(b"Checking for updates...\n").await.unwrap();

    let mut nSplit = name.as_ref().unwrap().split("-");

    let currV = nSplit.nth(1).unwrap().to_string();
    let currB:u64= nSplit.nth(0).unwrap().split(".").nth(0).unwrap().parse().unwrap();

    let remoteB = dlMgr::getLatBuild(&currV).await;

    if currB==remoteB {
        //No upd found.
        return name
    }
    stdout().write_all(b"Updates found, updating...\n").await.unwrap();
    fs::remove_file(name.unwrap()).unwrap();
    start_dl(Some(currV)).await.unwrap();
    Some(getSrvName().unwrap())

}

async fn start_dl(verOpt:Option<String>) -> Result<(),Box<dyn error::Error+Send+Sync>> {
    let mut lTask:JoinHandle<Result<(),Error>>;
    let mut dl:DlMgr;
    //Req user input until it provides a good one.
    loop {
        let mut toReqVer:String;
        if let Some(ref ver)=verOpt {
            toReqVer= ver.clone();
        } else {
            let mut stdout = stdout();
            let mut stdin = BufReader::new(stdin());
            stdout.write_all(b"Version to download (latest): ").await?;
            stdout.flush().await?;
            toReqVer=String::new();
            stdin.read_line(&mut toReqVer).await?;
            stdout.write_all(b"Getting version information...\n").await?;
        }
        dl = DlMgr::init(toReqVer);
        if let Err(e)=dl.fetch().await {
            stdout().write_all(format!("Error while requesting that version: {}\n",e).as_bytes()).await?;
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
    stdout().write_all(b"\n").await.unwrap();

    if !isCorrect {
        fs::remove_file("server.jar")?;
        stdout().write_all(b"Download hash mismatch! Press enter to exit...\n").await.unwrap();
        stdin().read_to_string(&mut String::new()).await?;
        exit(0)
    }
    stdout().write_all(b"Download hash verified.\n").await.unwrap();
    Ok(())
}
fn start_srv(name:String) {
    if let Err(e) = Command::new("java").arg("-jar").arg(name).arg("nogui").status() {
        eprintln!("Failed to start the server: {}", e);
    }
}
//Loop instead of recursion -> stackoverflow on too much bad values fixed.
fn accept_eula()->Result<bool,Error> {
    loop {
        print!("Do you agree to the eula? (https://aka.ms/MinecraftEULA) [Y/N] (Y): ");
        io::stdout().flush()?;
        let mut resp = String::new();
        io::stdin().read_line(&mut resp)?;
        match resp.as_str().trim() {
            "Y" | "y" | "" => {
                let mut file = File::create("eula.txt")?;
                file.write_all(b"eula=true")?;
                break Ok(true);
            },
            "N" | "n" => {
                println!("You will need to agree to the eula to continue.");
                break Ok(false)
            },
            _ => {
                println!("Incorrect answer.");
            }
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
