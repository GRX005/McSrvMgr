use std::fs::File;
use std::io;
use std::io::{read_to_string, stdin, Error, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

//Loop instead of recursion -> stackoverflow on too much bad values fixed.
pub fn accept_eula()->Result<bool,Error> {
    loop {
        print!("Do you agree to the eula? (https://aka.ms/MinecraftEULA) [Y/N] (Y): ");
        io::stdout().flush()?;
        let mut resp = String::new();
        stdin().read_line(&mut resp)?;
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
use tokio::io::stdin as asyncIn;
use tokio::io::stdout as asyncOut;
pub async fn getVer()->Result<String,Box<dyn std::error::Error+Send+Sync>> {
    let mut toReqVer = String::new();
    let mut stdout = asyncOut();
    let mut stdin = BufReader::new(asyncIn());
    stdout.write_all(b"Version to download (latest): ").await?;
    stdout.flush().await?;
    stdin.read_line(&mut toReqVer).await?;
    stdout.write_all(b"Getting version information...\n").await?;
    Ok(toReqVer)
}
//true = paper
pub fn getSrvType()->Result<bool,Error> {
    loop {
        print!("Velocity or Paper? [V/P]: ");
        io::stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        match buf.as_str().trim() {
            "V" | "v" => break Ok(false),
            "P" | "p" => break Ok(true),
            _ => println!("Incorrect answer.")
        }
    }
}