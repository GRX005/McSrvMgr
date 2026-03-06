#![allow(non_snake_case)]

mod dlMgr;
mod usrInp;

use crate::dlMgr::DlMgr;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::stdin;
use std::process::{exit, Command};
use std::{error, fs};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

pub enum DlMsg {
    StartWithSize(u64),
    Chunk(u64)
}

fn main() -> Result<(),Box<dyn error::Error+Send+Sync>> {
    println!("Minecraft Server Manager - V1.0");
    let mut srv = checkLat();
    if srv.is_none() {
        let isPaper = usrInp::getSrvType()?;
        start_dl(None, isPaper)?;
        srv = getSrvName();
    }
    let isV = srv.as_ref().unwrap().contains("V");
    if !fs::exists("eula.txt")? && !isV {
        if !usrInp::accept_eula()? {
            fs::remove_file(getSrvName().unwrap())?;
            exit(0);
        }
    }
    start_srv(srv.unwrap(),isV);
    Ok(())
}

fn getSrvName() -> Option<String> {
    fs::read_dir(".").unwrap().find_map(|v| {
        v.unwrap().file_name().to_str().filter(|s| s.starts_with("server")).filter(|s| s.ends_with(".jar")).map(str::to_owned)
    })
}
fn checkLat() -> Option<String> {
    let name = getSrvName();
    if name.is_none() {
        return None;
    }
    println!("Checking for updates...");

    let mut nSplit = name.as_ref().unwrap().split("-");

    let mut currV = nSplit.nth(1).unwrap().to_string();
    let currB:u64= nSplit.nth(0).unwrap().split(".").nth(0).unwrap().parse().unwrap();
    let isPaper = !name.as_ref()?.contains("V");
    let remoteB = dlMgr::getLatBuild(&mut currV,isPaper).expect("Failed to get latest build!");

    if currB==remoteB {
        //No upd found.
        return name
    }
    println!("Server jar out of date!");
    fs::remove_file(name.unwrap()).unwrap();
    start_dl(Some(currV),isPaper).unwrap();
    Some(getSrvName().unwrap())

}
fn start_dl(mut verOpt:Option<String>, isPaper:bool) -> Result<(),Box<dyn error::Error+Send+Sync>> {
    let mut dl:DlMgr;
    //Req user input until it provides a good one.
    loop {
        let toReqVer:String;
        if let Some(ver)=verOpt.take() {
            toReqVer= ver;
        } else {
            toReqVer= usrInp::getVer()?;
        }
        dl = DlMgr::init(toReqVer, isPaper);
        if let Err(e)=dl.fetch() {
            println!("{}", format!("Error while requesting that version: {}\n",e));
        } else {
            break;
        }
    }
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    dl = rt.block_on(async {
        let (tx, rx) = mpsc::channel(32);
        let dl = tokio::spawn(dl.download(tx));
        let prog = tokio::spawn(dlProgress(rx));
        let (dlRes, _) = tokio::join!(dl,prog);
        return dlRes?
    })?;

    println!("Cheking server integrity...");
    let isCorrect = dl.verify()?;

    if !isCorrect {
        fs::remove_file(getSrvName().unwrap())?;
        println!("Server integrity FAIL! Press enter to try downloading again...");
        stdin().read_line(&mut String::new())?;
        exit(0);//TODO DO RETRY LOOP
    }
//TODO Might bug into other texts??
    println!("Server integrity PASS!");
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

async fn dlProgress(mut rx:Receiver<DlMsg>) {
    let mut pb = ProgressBar::hidden();
    // Loop ends automatically when the channel `tx` closes
    while let Some(msg) = rx.recv().await {
        match msg {
            DlMsg::StartWithSize(size) => {
                pb = ProgressBar::new(size);
                pb.set_style(
                    ProgressStyle::with_template(
                        "{msg} {percent:>3}% [{bar:50}] {bytes:>10}/{total_bytes:<10}"
                    )
                        .unwrap()
                        .progress_chars("━╸ "),
                );
                pb.set_message("Downloading server...");
            }
            DlMsg::Chunk(size) => pb.inc(size),
        }
    }
    pb.finish_with_message("Done!");
}

