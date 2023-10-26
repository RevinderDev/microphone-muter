use chrono::Local;

use std::ptr::{self};
use std::sync::Mutex;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::{
    core::Error,
    Win32::{
        Media::Audio::{
            eCapture, eMultimedia, Endpoints::IAudioEndpointVolume, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
        },
    },
};

use clap::Parser;
use hookmap::{device, prelude::*};
use std::process;
use std::sync::mpsc;
use std::thread;

use tray_item::{IconSource, TrayItem};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    device_name: Option<String>,
}

enum Message {
    Quit,
    Test,
}

struct WrappedIAudioEndpointVolumePointer(IAudioEndpointVolume);
unsafe impl Send for WrappedIAudioEndpointVolumePointer {}

struct SafeIAudioEndpointVolume {
    mutex: Mutex<WrappedIAudioEndpointVolumePointer>,
}

unsafe fn get_device_name(device: &IMMDevice) -> Result<String, Error> {
    let property_store = device.OpenPropertyStore(STGM_READ)?;
    return Ok(
        PropVariantToStringAlloc(&property_store.GetValue(&PKEY_Device_FriendlyName)?)?
            .to_string()?,
    );
}

unsafe fn get_microphone(searched_device_name: Option<String>) -> Result<IMMDevice, Error> {
    CoInitializeEx(None, COINIT_MULTITHREADED)?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    if let Some(searched_device_name) = searched_device_name {
        let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
        let collection_size = collection.GetCount()?;
        for index in 0..collection_size {
            let device = collection.Item(index)?;
            if get_device_name(&device).unwrap() == searched_device_name {
                return Ok(device);
            }
        }
        panic!("Couldn't find desired microphone.");
    } else {
        return Ok(enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia)?);
    };
}

fn main() -> Result<(), Error> {
    let mut tray = TrayItem::new("Microphone Muter", IconSource::Resource("aa-exe-icon")).unwrap();

    let cli = Cli::parse();
    let audio_endpoint = unsafe {
        let microphone = get_microphone(cli.device_name).unwrap();
        let device_name = get_device_name(&microphone).unwrap();
        println!("✅ Found microphone '{}'", device_name);
        tray.add_label(format!("Selected device: {device_name}").as_str())
            .unwrap();

        let audio_endpoint = microphone.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
        Ok::<IAudioEndpointVolume, Error>(audio_endpoint)
    }
    .unwrap();

    let safe_audio_endpoint = SafeIAudioEndpointVolume {
        mutex: Mutex::new(WrappedIAudioEndpointVolumePointer(audio_endpoint)),
    };

    let (tx, rx) = mpsc::sync_channel(2);

    let quit_receiver = tx.clone();
    tray.inner_mut().add_separator().unwrap();
    tray.add_menu_item("Quit", move || {
        quit_receiver.send(Message::Quit).unwrap();
    })
    .unwrap();
    let test = tx.clone();
    let mut hotkey = Hotkey::new();
    hotkey
        .register(
            Context::new()
                .modifiers(buttons!(LAlt))
                .native_event_operation(NativeEventOperation::Block),
        )
        .on_press(Button::SideButton2, move |_| {
            let audio_endpoint = &safe_audio_endpoint.mutex.lock().unwrap().0;
            let local_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            unsafe {
                let muting_action = !audio_endpoint.GetMute().unwrap();
                let muting_message = match muting_action.into() {
                    true => "🔇",
                    false => "🔊",
                };
                audio_endpoint
                    .SetMute(!audio_endpoint.GetMute().unwrap(), ptr::null())
                    .unwrap();
                println!(
                    "{}",
                    format!("[🕖 {local_time} 🕖] Microphone is {muting_message}")
                );
                test.send(Message::Test).unwrap();
            }
        });

    // TODO: Swap icon when it mutes the microphone.
    // TODO: Select microphone through UI.
    // TODO: Move everything to much easier threading model.

    thread::spawn(move || {
        hotkey.install();
    });
    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Received Quit");
                tray.inner_mut().quit();
                tray.inner_mut().shutdown().unwrap();
                process::exit(0);
            }
            Ok(Message::Test) => {
                println!("Received Test");
            }
            _ => {}
        }
    }
}
