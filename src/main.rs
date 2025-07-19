#![allow(non_snake_case)]

mod dlMgr;
mod usrInp;

use crate::dlMgr::DlMgr;
use std::io::Error;
use std::process::{exit, Command};
use std::time::Duration;
use std::{error, fs, io};
use tokio::io::{stdin, AsyncReadExt};
use tokio::io::{stdout, AsyncWriteExt};
use tokio::task::{spawn_blocking, JoinHandle};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn error::Error+Send+Sync>> {
    stdout().write_all(b"Minecraft Server Manager - V1.0\n").await?;
    let mut srv = checkLat().await;
    if srv.is_none() {
        let isPaper = spawn_blocking(||->bool {usrInp::getSrvType().unwrap()}).await?;
        start_dl(None, isPaper).await?;
        srv = getSrvName().await;
    }
    let isV = srv.as_ref().unwrap().contains("V");
    spawn_blocking(move || -> Result<(), Error> {
        if !fs::exists("eula.txt")? && !isV {
            if !usrInp::accept_eula()? {
                exit(0);
            }
        }
        Ok(())
    }).await??;
    spawn_blocking(move || start_srv(srv.unwrap(),isV)).await?;

    Ok(())
}

async fn getSrvName() -> Option<String> {
    spawn_blocking(|| {
        fs::read_dir(".").unwrap().find_map(|v| {
            v.unwrap().file_name().to_str().filter(|s| s.starts_with("server")).filter(|s| s.ends_with(".jar")).map(str::to_owned)
        })
    }).await.unwrap()
}
async fn checkLat() -> Option<String> {
    let name = getSrvName().await;
    if name.is_none() {
        return None;
    }
    stdout().write_all(b"Checking for updates...\n").await.unwrap();

    let mut nSplit = name.as_ref().unwrap().split("-");

    let mut currV = nSplit.nth(1).unwrap().to_string();
    let currB:u64= nSplit.nth(0).unwrap().split(".").nth(0).unwrap().parse().unwrap();
    let isPaper = !name.as_ref()?.contains("V");
    let remoteB = dlMgr::getLatBuild(&mut currV,isPaper).await;

    if currB==remoteB {
        //No upd found.
        return name
    }
    stdout().write_all(b"Server jar out of date!\n").await.unwrap();
    tokio::fs::remove_file(name.unwrap()).await.unwrap();
    start_dl(Some(currV),isPaper).await.unwrap();
    Some(getSrvName().await.unwrap())

}

async fn start_dl(mut verOpt:Option<String>, isPaper:bool) -> Result<(),Box<dyn error::Error+Send+Sync>> {
    let mut dl:DlMgr;
    //Req user input until it provides a good one.
    loop {
        let toReqVer:String;
        if let Some(ver)=verOpt.take() {
            toReqVer= ver;
        } else {
            toReqVer= usrInp::getVer().await?;
        }
        dl = DlMgr::init(toReqVer, isPaper);
        if let Err(e)=dl.fetch().await {
            stdout().write_all(format!("Error while requesting that version: {}\n",e).as_bytes()).await?;
        } else {
            break;
        }
    }
    let mut lTask:JoinHandle<Result<(),Error>>;
    lTask=tokio::spawn(loading());
    dl.download().await?;
    lTask.abort();
    stdout().write_all(b"\n").await?;

    lTask=tokio::spawn(hashLoading());
    let isCorrect = dl.verify().await?;
    lTask.abort();
    stdout().write_all(b"\n").await?;

    if !isCorrect {
        tokio::fs::remove_file("server.jar").await?;
        stdout().write_all(b"Download hash mismatch! Press enter to exit...\n").await?;
        stdin().read_to_string(&mut String::new()).await?;
        exit(0)
    }
    stdout().write_all(b"Download hash verified.\n").await?;
    Ok(())
}
//Start with the recommended flags by paper.
fn start_srv(name:String, isV:bool) {
    let optArgs:std::str::Split<&str>;
    if isV {//Velocity
        optArgs="-XX:+UseG1GC -XX:G1HeapRegionSize=4M -XX:+UnlockExperimentalVMOptions -XX:+ParallelRefProcEnabled -XX:+AlwaysPreTouch -XX:MaxInlineLevel=15 -jar".split(" ");
    } else {//Paper
        optArgs="-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -jar".split(" ");
    }
    if let Err(e) = Command::new("java").args(optArgs).arg(name).arg("nogui").status() {
        eprintln!("Failed to start the server: {}", e);
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
