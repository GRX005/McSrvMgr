#![allow(non_snake_case)]

use std::fs::File;
use std::io::{stdin, Write};
use std::process::Command;
use std::time::Duration;
use std::{fs, io};
use tokio::fs::File as AsyncFile;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    if !fs::exists("server.jar").expect("Err checking server.jar exists") {//Dl in not exists
        let lTask =tokio::spawn(loading());
        tokio::spawn(dl()).await??;
        lTask.abort();
        println!();
        if !accept_eula() {
            return Ok(())
        }
    }
    if !fs::exists("eula.txt").expect("Err checking eula.txt exists") {
        if !accept_eula() {
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

fn accept_eula()->bool {
    print!("Do you agree to the eula? (https://aka.ms/MinecraftEULA) [Y/N]: ");
    io::stdout().flush().expect("Err buffer flush");
    let mut resp = String::new();
    stdin().read_line(&mut resp).expect("Error while reading your response.");
    match resp.as_str().trim() {
        "Y" | "y" => {
            let mut file = File::create("eula.txt").expect("Err creating eula file");
            file.write_all(b"eula=true").expect("Err writing eula");
            true
        },
        "N" | "n" => {
            println!("You will need to agree to the eula to continue.");
            false
        },
        _ => {
            println!("Incorrect answer.");
            accept_eula()
        }
    }
}

async fn dl()-> Result<(), Box<dyn std::error::Error+Send+Sync>> {
    let paper = "https://fill-data.papermc.io/v1/objects/5554d04f7b72cf9776843d7d600dfa72062ad4e9991dbcf6d7d47bdd58cead9f/paper-1.21.7-16.jar";
    let mut dl_file = reqwest::get(paper).await?.error_for_status()?;
    let mut disk = AsyncFile::create("server.jar").await?;
    while let Some(elem)= dl_file.chunk().await? {
        disk.write_all(&elem).await?;
    }
    Ok(())
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
