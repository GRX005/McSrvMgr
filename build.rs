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

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_language(0x0409);
        res.set("LegalCopyright",  "Copyright © 2025-2026 _1ms");
        res.set("OriginalFilename", "McSrvMgr.exe");
        res.set("FileDescription", "Minecraft Server Manager");
        res.compile().unwrap();
    }
}