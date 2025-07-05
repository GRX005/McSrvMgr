#![allow(non_snake_case)]

use std::io::{stdin, Write};
use std::process::Command;
use std::time::Duration;
use std::{fs, io};
use tokio::fs::File as AsyncFile;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    println!("Minecraft Server Manager - V1.0");
    if !fs::exists("server.jar").unwrap() {//Dl in not exists
        let lTask =tokio::spawn(loading());
        tokio::spawn(dl()).await??;
        lTask.abort();
        println!()
    }

    start_srv();

    let eulaP = Path::new("eula.txt");
    let eulaC= fs::read_to_string(eulaP).expect("Err reading eula");
    if eulaC.contains("eula=false") && !accept_eula(eulaP,eulaC.split("\n").collect()) {
        return Ok(());
    }
    start_srv();
    Ok(())
}

fn start_srv() {
    if let Err(e) = Command::new("java").arg("-jar").arg("server.jar").arg("--nogui").status() {
        println!("Failed to start the server: {}", e);
    }
}

use std::path::Path;

fn accept_eula(fPath:&Path, mut cont:Vec<&str>) ->bool {
    print!("Do you agree to the eula? [Y/N]: ");
    io::stdout().flush().expect("Err buffer flush");
    let mut resp = String::new();
    stdin().read_line(&mut resp).expect("Error while reading your response.");
    match resp.as_str().trim() {
        "Y" | "y" => {
            if let Some(eula) = cont.last_mut() {
                *eula = "eula=true";
            }
            fs::write(fPath, cont.join("\n")).expect("Err writing eula");
            true
        },
        "N" | "n" => {
            println!("You will need to agree to the eula to continue.");
            false
        },
        _ => {
            println!("Incorrect answer.");
            accept_eula(fPath,cont)
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
