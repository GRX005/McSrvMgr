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

use console::style;
use std::fs::File;
use std::io::{stdin, stdout, Error, Write};

//Loop instead of recursion -> stackoverflow on too much bad values fixed.
pub fn accept_eula()->Result<bool,Error> {
    loop {
        let Y = style("Y").bold();
        print!("Do you agree to the eula? ({}) [{}/{}] ({}): ",style("https://aka.ms/MinecraftEULA").cyan(),Y,style("N").bold(),Y);
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
                println!("{}",style("You will need to agree to the eula to continue.").yellow());
                break Ok(false)
            },
            _ => println!("{}",style("Invalid answer!").red())
        }
    }
}
pub fn getVer()->Result<String,Error> {
    let mut input;
    loop {
        print!("Version to download ({}): ",style("latest").bold());
        stdout().flush()?;
        input = String::new();
        stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if !input.bytes().any(|b| !matches!(b, b'0'..=b'9' | b'.' | b'-' | b'a'..=b'z' | b'A'..=b'Z')) {
            break
        }
        println!("{}",style("Invalid answer!").red())
    }
    println!("Getting version information...");
    Ok(input)
}
//true = paper
pub fn getSrvType()->Result<bool,Error> {
    loop {
        print!("Velocity or Paper? [{}/{}]: ",style("V").bold(),style("P").bold());
        stdout().flush()?;
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        match buf.as_str().trim() {
            "V" | "v" => break Ok(false),
            "P" | "p" => break Ok(true),
            _ => println!("{}",style("Invalid answer!").red())
        }
    }
}