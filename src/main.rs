// jkcoxson
// All in one tool for activating JIT on iOS devices

use std::{
    env,
    fs::{self, File},
    io,
};

use config::Device;
use plist;
use rusty_libimobiledevice::libimobiledevice;
mod config;
mod install;
mod user_input;

fn main() {
    println!("#################");
    println!("## JIT Shipper ##");
    println!("##  jkcoxson   ##");
    println!("#################\n\n");

    let x = rusty_libimobiledevice::libimobiledevice::idevice_get_device_list_extended().unwrap();
    let devices = x.0;
    let count = x.1;
    println!("Found {} devices", count);
    let udid = devices[0].udid.clone();
    let ntwk = if devices[0].conn_type.clone() == 1 {
        false
    } else {
        true
    };
    for i in devices {
        println!("Device udid: {:?}", i.udid);
    }
    let dev =
        rusty_libimobiledevice::libimobiledevice::idevice_new_with_options(udid, ntwk).unwrap();

    println!("Connected to device: {:?}", dev.get_udid());

    let debug_cli =
        libimobiledevice::debugserver_client_start_service(dev, "Your mom part 2".to_string())
            .unwrap();

    let command =
        libimobiledevice::debugserver_command_new("QSetMaxPacketSize:".to_string(), 2).unwrap();
    let res = libimobiledevice::debugserver_client_send_command(debug_cli, command).unwrap();
    if res != "OK" {
        panic!("Failed to set max packet size");
    }

    todo!("The rest of this project needs to be translated to the lib");

    // Get home directory
    let home_dir = dirs::home_dir().unwrap();
    // Detect if home_dir/libimobiledevice is present
    let libimobiledevice_path = home_dir.join("libimobiledevice");
    if !libimobiledevice_path.exists() {
        // If not, create it
        fs::create_dir(libimobiledevice_path).expect("Failed to create libimobiledevice directory");
    }
    // Change directory to libimobiledevice
    env::set_current_dir(libimobiledevice_path).expect("Failed to change directory");
    ui_loop();
}

fn ui_loop() {
    loop {
        // match choose_device() {
        //     Some(device) => {
        //         let _dmg_path = get_ios_dmg(&device);
        //         let pkg_name = choose_app(&device);
        //         match device.run_app(pkg_name) {
        //             true => {
        //                 println!("Successfully launched the app");
        //             }
        //             false => {
        //                 println!("Failed to launch the app");
        //             }
        //         }
        //     }
        //     None => {
        //         println!("No devices detected, connect a device and then press enter");
        //         std::io::stdin().read_line(&mut String::new()).unwrap();
        //     }
        // }
    }
}

fn choose_device() -> Option<libimobiledevice::idevice_info> {
    todo!()
}

fn get_device_list() -> Option<Vec<Device>> {
    let devices = rusty_libimobiledevice::libimobiledevice::idevice_get_device_list_extended()
        .expect("Failed to get device list");
    // If no devices are detected, return None
    if devices.1 == 0 {
        return None;
    }
    //let mut device_list = Vec::new();
    for i in devices.0 {
        let idev = libimobiledevice::idevice_new_with_options(
            i.udid,
            if i.conn_type == 1 { false } else { true },
        )
        .expect("Failed to fetch device information");
        let lock_cli =
            libimobiledevice::lockdownd_client_new_with_handshake(idev, "JIT Shipper".to_string())
                .expect("Failed to create lockdownd client");
        let values =
            libimobiledevice::lockdownd_get_value(lock_cli).expect("Failed to get product version");
        // Get plist from values
        //let dev_info = plist::from_bytes(values.as_bytes()).expect("Failed to parse plist");
        //println!("{:?}", dev_info);
    }

    todo!()
}

fn get_ios_dmg(device: &Device) -> String {
    // Get directory
    let home_dir = dirs::home_dir().unwrap();
    let libimobiledevice_path = home_dir.join("libimobiledevice");
    let ios_path = &libimobiledevice_path.join(device.version.clone());
    // Check if directory exists
    if ios_path.exists() {
        // Check if DMG exists
        let ios_dmg = ios_path.join("DeveloperDiskImage.dmg");
        if ios_dmg.exists() {
            return ios_dmg.to_str().unwrap().to_string();
        } else {
            // Remove iOS directory
            std::fs::remove_dir_all(ios_path).unwrap();
        }
    }
    // Download versions.json from GitHub
    println!("Downloading iOS dictionary...");
    let url = "https://raw.githubusercontent.com/jkcoxson/jit_shipper/master/versions.json";
    let response = reqwest::blocking::get(url).expect("Failed to download iOS version library");
    let contents = response.text().expect("Failed to read iOS version library");
    // Parse versions.json
    let versions: serde_json::Value = serde_json::from_str(&contents).unwrap();
    // Get DMG url
    let ios_dmg_url = match versions.get(device.version.as_str()) {
        Some(x) => x.as_str().unwrap().to_string(),
        None => panic!(
            "\nCould not find {} from the library. Check back later!\n",
            device.version
        ),
    };
    // Download DMG zip
    println!("Downloading iOS {} DMG...", device.version);
    let mut resp = reqwest::blocking::get(ios_dmg_url).expect("Unable to download DMG");
    let mut out = File::create("dmg.zip").expect("Failed to create zip");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
    // Unzip zip
    let mut dmg_zip = zip::ZipArchive::new(File::open("dmg.zip").unwrap()).unwrap();
    dmg_zip.extract(libimobiledevice_path).unwrap();
    // Remove zip
    std::fs::remove_file("dmg.zip").unwrap();
    // Return DMG path
    let ios_dmg = ios_path.join("DeveloperDiskImage.dmg");
    ios_dmg.to_str().unwrap().to_string()
}

fn choose_app(device: &Device) -> String {
    println!("Fetching apps installed on device...");
    let apps = device.app_scan();
    let mut options = vec![];
    for (key, _) in &apps {
        options.push(key.clone().replace("\"", ""));
    }
    options.sort();
    let options: Vec<&str> = options.iter().map(|x| x.as_str()).collect();
    let choice = user_input::multi_input("Choose an app", options.as_slice());
    return apps
        .get(format!("\"{}\"", choice).as_str())
        .unwrap()
        .clone();
}
