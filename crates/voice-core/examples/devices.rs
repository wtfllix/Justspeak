//! Real native device enumeration; no synthesized devices and no room connection.
#[tokio::main]
async fn main() {
    match voice_core::VoiceCore::default().devices().await {
        Ok(devices) => {
            println!("Native recording devices: {}", devices.len());
            for device in devices {
                println!("{}: {}", device.id, device.name);
            }
        }
        Err(message) => {
            eprintln!("Native device enumeration failed: {message}");
            std::process::exit(1);
        }
    }
}
