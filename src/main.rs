/*
    This file is part of the McSrvMgr project, licensed under the
    GNU General Public License v3.0

    Copyright (C) 2025-2026 _1ms (GRX005)

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

#![allow(non_snake_case)]

mod dlMgr;
mod usrInp;

use crate::dlMgr::DlMgr;
use console::{style, Term};
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::{exit, Command};
use ureq::Agent;

fn main() -> Result<(),Box<dyn Error>> {
    println!("Minecraft Server Manager - v{}",env!("CARGO_PKG_VERSION"));
    Term::stdout().set_title("Minecraft Server Manager");
    let netAgent = getAgent();
    let mut srv = checkLat(&netAgent);
    if srv.is_none() {
        let isPaper = usrInp::getSrvType()?;
        start_dl(None, isPaper, netAgent)?;
        srv = getSrvName();
    }
    let isV = srv.as_ref().unwrap().contains("V");
    if !fs::exists("eula.txt")? && !isV {
        if !usrInp::accept_eula()? {
            fs::remove_file(getSrvName().unwrap())?;
            exit(0);
        }
    }
    start_srv(srv.unwrap());
    Ok(())
}

fn getSrvName() -> Option<String> {
    fs::read_dir(".").unwrap().find_map(|v| {
        v.unwrap().file_name().to_str().filter(|s| s.ends_with(".jar") && s.starts_with("server")).map(str::to_owned)
    })
}
fn checkLat(netAgent:&Agent) -> Option<String> {
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

    let remoteB = dlMgr::getLatBuild(&mut currV,isPaper,netAgent).unwrap();

    if currB==remoteB {
        //No upd found.
        return name
    }
    println!("{}", style("Server jar out of date!").yellow());
    fs::remove_file(name.unwrap()).unwrap();
    start_dl(Some(currV),isPaper, netAgent.clone()).unwrap();
    Some(getSrvName().unwrap())

}
fn start_dl(mut verOpt:Option<String>, isPaper:bool, netAgent:Agent) -> Result<(),Box<dyn Error>> {
    let mut dl:DlMgr;
    //Req user input until it provides a good one.
    loop {
        let toReqVer:String;
        if let Some(ver)=verOpt.take() {
            toReqVer = ver;
        } else {
            toReqVer = usrInp::getVer()?;
        }
        dl = DlMgr::init(toReqVer, isPaper, netAgent.clone());
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
fn start_srv(name:String) {
    //Args disabled as there is no consistent info on what type should be used and when, specially on modern java.
    if let Err(e) = Command::new("java").arg("-XX:+AlwaysPreTouch").arg("-jar").arg(name).arg("nogui").status() {
        eprintln!("{}", style(format!("Failed to start the server: {}", e)).red());
    }
}

fn getAgent() -> Agent {
    Agent::config_builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), " (https://github.com/GRX005/McSrvMgr)"))
        .https_only(true)
        .http_status_as_error(false)
        .build()
        .new_agent()
}
