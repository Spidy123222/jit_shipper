// jkcoxson
// All in one tool for activating JIT on iOS devices

use std::{convert::TryInto, env, fmt::format, fs::File, io, ptr::NonNull};

use config::Device;
mod config;
mod install;
mod user_input;

fn main() {
    println!("#################");
    println!("## JIT Shipper ##");
    println!("##  jkcoxson   ##");
    println!("#################\n\n");

    // Get home directory
    let home_dir = dirs::home_dir().unwrap();
    // Detect if home_dir/libimobiledevice is present
    let libimobiledevice_path = home_dir.join("libimobiledevice");
    if libimobiledevice_path.exists() {
        println!("libimobiledevice is installed");
        ui_loop();
    } else {
        println!("libimobiledevice is NOT installed");
        if user_input::yes_no_prompt("Would you like to install libimobiledevice?") {
            install::install();
        } else {
            println!("Exiting...");
            return;
        }
        ui_loop();
    }
}

fn ui_loop() {
    // Get directory
    let home_dir = dirs::home_dir().unwrap();
    let libimobiledevice_path = home_dir.join("libimobiledevice");
    // Change path to libimobiledevice
    env::set_current_dir(&libimobiledevice_path).unwrap();

    loop {
        match choose_device() {
            Some(device) => {
                let dmg_path = get_ios_dmg(&device);
            }
            None => {
                println!("No devices detected, connect a device and then press enter");
                std::io::stdin().read_line(&mut String::new()).unwrap();
            }
        }
    }
}

fn choose_device() -> Option<Device> {
    let devices = Device::device_scan();
    let mut options = vec![];
    for device in &devices {
        options.push(device.name.clone());
    }
    // Check if there are any devices
    if options.len() == 0 {
        return None;
    }
    // Convert strings to str array
    let options: Vec<&str> = options.iter().map(|x| x.as_str()).collect();
    // Convert options to an array
    let options = options.as_slice();
    let device_name = user_input::multi_input("Choose a device", options);

    for device in devices {
        if device.name == device_name {
            return Some(device);
        }
    }

    panic!("You shouldn't see this error message, this is just to make the compiler happy.");
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
    let mut resp = reqwest::blocking::get(ios_dmg_url).expect("Unable to download DMG");
    let mut out = File::create("dmg.zip").expect("Failed to create zip");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
    // Unzip zip
    let mut dmg_zip = zip::ZipArchive::new(File::open("dmg.zip").unwrap()).unwrap();
    dmg_zip.extract(ios_path).unwrap();
    // Remove zip
    std::fs::remove_file("dmg.zip").unwrap();
    // Return DMG path
    let ios_dmg = ios_path.join("DeveloperDiskImage.dmg");
    ios_dmg.to_str().unwrap().to_string()
}
