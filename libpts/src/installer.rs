use std::fs;
use std::io;
use std::process::Command;

use crate::wine::Wine;

const SERVER: &[u8] = include_bytes!(env!("SERVER_PATH"));

pub const PTS_PATH: &'static str = "pts";

// Directory name where the installer extract his
// files with the `/extract` flag
const INSTALLER_EXTRACT_DIR: &'static str = "BE36A8D";

pub fn install_pts(wine: &Wine, mut installer_src: impl io::Read) {
    let drive_c = wine.drive_c();
    let installer = drive_c.join("installer.exe");
    let tmp = drive_c.join("tmp");
    let system32 = drive_c.join("windows/system32");
    let pts = drive_c.join(PTS_PATH);

    let mut installer_dst = fs::File::create(installer).expect("Create Installer");

    io::copy(&mut installer_src, &mut installer_dst).expect("Write Installer");

    fs::create_dir(&tmp).expect("Create dir");

    // TODO: check status
    wine.command("installer.exe", true)
        .arg("/extract")
        .arg(r"C:\tmp")
        .status()
        .expect("Installer");

    // TODO: check status
    Command::new("cabextract")
        .current_dir(&tmp)
        .arg("Visual C++ 2008 Redistributable/vcredist_x86.exe")
        .arg("-F")
        .arg("vc_red.cab")
        .status()
        .expect("cabextract");

    // TODO: check status
    Command::new("cabextract")
        .current_dir(&tmp)
        .arg("vc_red.cab")
        .status()
        .expect("cabextract");

    fs::rename(tmp.join("nosxs_mfc90.dll"), system32.join("mfc90.dll")).expect("Rename failed");

    fs::rename(tmp.join(INSTALLER_EXTRACT_DIR), &pts).expect("Rename failed");

    fs::remove_dir_all(tmp).expect("Remove failed");

    let pixitx =
        fs::read_dir(pts.join("bin/Bluetooth/PIXITX")).expect("Failed to read pixitx directory");
    let picsx =
        fs::read_dir(pts.join("bin/Bluetooth/PICSX")).expect("Failed to read picsx directory");

    for entry in pixitx.chain(picsx) {
        let path = entry.unwrap().path();

        if let Some(extension) = path.extension() {
            let new = path.with_extension(extension.to_ascii_lowercase());
            fs::rename(path, new).expect("Rename failed");
        }
    }
}

pub fn is_pts_installation_needed(wine: &Wine) -> bool {
    !wine.drive_c().join(PTS_PATH).exists()
}

pub fn install_server(wine: &Wine) -> io::Result<()> {
    let mut path = wine.drive_c().join(PTS_PATH);
    path.push("bin");
    path.push("server.exe");

    fs::write(path, SERVER)
}
