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