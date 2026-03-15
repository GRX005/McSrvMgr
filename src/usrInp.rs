use std::fs::File;
use std::io::{stdin, stdout, Error, Write};

//Loop instead of recursion -> stackoverflow on too much bad values fixed.
pub fn accept_eula()->Result<bool,Error> {
    loop {
        print!("Do you agree to the eula? (https://aka.ms/MinecraftEULA) [Y/N] (Y): ");
        stdout().flush()?;
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

pub fn getVer()->Result<String,Error> {
    print!("Version to download (latest): ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    println!("Getting version information...");
    Ok(input)
}
//true = paper
pub fn getSrvType()->Result<bool,Error> {
    loop {
        print!("Velocity or Paper? [V/P]: ");
        stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        match buf.as_str().trim() {
            "V" | "v" => break Ok(false),
            "P" | "p" => break Ok(true),
            _ => println!("Incorrect answer.")
        }
    }
}