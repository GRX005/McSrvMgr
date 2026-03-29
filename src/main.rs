#![allow(non_snake_case)]

mod dlMgr;
mod usrInp;

use crate::dlMgr::DlMgr;
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::{exit, Command};
use console::{style, Term};

fn main() -> Result<(),Box<dyn Error>> {
    println!("Minecraft Server Manager - v{}",env!("CARGO_PKG_VERSION"));
    Term::stdout().set_title("Minecraft Server Manager");
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
        v.unwrap().file_name().to_str().filter(|s| s.ends_with(".jar") && s.starts_with("server")).map(str::to_owned)
    })
}
fn checkLat() -> Option<String> {
    let name = getSrvName();
    if name.is_none() {
        return None;
    }
    println!("Checking for updates...");
    //TODO Better error management?
    //name for ex.: server-1.21.11-127.jar
    let mut nSplit = name.as_ref().unwrap().split('_');
    let isPaper = !nSplit.next().unwrap().contains('V');
    //Like 1.21.11
    let mut currV = nSplit.next().unwrap().to_string();
    //Like 127, from "127.jar"
    let currB:u64= nSplit.next().unwrap().split('.').next().unwrap().parse().unwrap();

    let remoteB = dlMgr::getLatBuild(&mut currV,isPaper).unwrap();

    if currB==remoteB {
        //No upd found.
        return name
    }
    println!("{}", style("Server jar out of date!").yellow());
    fs::remove_file(name.unwrap()).unwrap();
    start_dl(Some(currV),isPaper).unwrap();
    Some(getSrvName().unwrap())

}
fn start_dl(mut verOpt:Option<String>, isPaper:bool) -> Result<(),Box<dyn Error>> {
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
            eprintln!("{}",style(format!("Error while requesting that version: {}",e)).red());
        } else {
            break;
        }
    }
    loop {
        if dl.downloadAndVerify()? {
            break
        }
        //If integrity verified, exit.
        fs::remove_file(getSrvName().unwrap())?;
        println!("Server integrity {}!",style("FAIL").red());
        print!("Press enter to try downloading it again...");
        stdout().flush()?;
        stdin().read_line(&mut String::new())?;
    }
    println!("Server integrity {}!",style("PASS").green());
    Ok(())
}
//Start with the recommended flags by paper.
fn start_srv(name:String, isV:bool) {
    let optArgs:std::str::Split<&str>;
    if isV {//Velocity
        optArgs="-XX:+AlwaysPreTouch -XX:+ParallelRefProcEnabled -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1HeapRegionSize=4M -XX:MaxInlineLevel=15 -jar".split(" ");
    } else {//Paper
        optArgs="-XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+ParallelRefProcEnabled -XX:+PerfDisableSharedMem -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1HeapRegionSize=8M -XX:G1HeapWastePercent=5 -XX:G1MaxNewSizePercent=40 -XX:G1MixedGCCountTarget=4 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1NewSizePercent=30 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:MaxGCPauseMillis=200 -XX:MaxTenuringThreshold=1 -XX:SurvivorRatio=32 -jar".split(" ");
    }
    if let Err(e) = Command::new("java").args(optArgs).arg(name).arg("nogui").status() {
        eprintln!("{}", style(format!("Failed to start the server: {}", e)).red());
    }
}

