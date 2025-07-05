#![allow(non_snake_case)]

use std::time::Duration;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error+Send+Sync>> {
    println!("Minecraft Server Manager - V1.0");

    let lTask =tokio::spawn(loading());
    tokio::spawn(dl()).await??;
    lTask.abort();
    Ok(())

}

async fn dl()-> Result<(), Box<dyn std::error::Error+Send+Sync>> {
    let paper = "https://fill-data.papermc.io/v1/objects/5554d04f7b72cf9776843d7d600dfa72062ad4e9991dbcf6d7d47bdd58cead9f/paper-1.21.7-16.jar";
    let mut dl_file = reqwest::get(paper).await?.error_for_status()?;
    let mut disk = File::create("server.jar").await?;
    while let Some(elem)= dl_file.chunk().await? {
        disk.write_all(&elem).await?;
    }
    Ok(())
}

async fn loading()-> std::io::Result<()> {
    let mut out = stdout();
    out.write_all(b"Downloading server").await?;
    out.flush().await?;
    loop {
        out.write_all(b".").await?;
        out.flush().await?;
        sleep(Duration::from_millis(200)).await;
    }
}