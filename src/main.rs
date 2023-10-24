use chrono::Local;
use std::os::raw::c_void;
use std::ptr::{self, NonNull};
use std::sync::{Arc, Mutex};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::{
    core::Error,
    Win32::{
        Foundation::{FALSE, TRUE},
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
use hookmap::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    device_name: Option<String>,
}

struct WrappedIAudioEndpointVolumePointer(IAudioEndpointVolume);
unsafe impl Send for WrappedIAudioEndpointVolumePointer {}

struct SafeIAudioEndpointVolume {
    mutex: Mutex<WrappedIAudioEndpointVolumePointer>,
}

unsafe fn get_microphone(device_name: Option<String>) -> Result<IMMDevice, Error> {
    CoInitializeEx(None, COINIT_MULTITHREADED)?;
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    // TODO: here this should return device
    if let Some(device_name) = device_name {
        println!("Test")
    } else {
        println!("No microphone specified, fetching default");
        return Ok(enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia)?);
    }

    // TODO: Print out it's name return
    let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
    println!("Found devices: {}", collection.GetCount()?);
    let device = collection.Item(0)?;
    println!("Device id: {}", device.GetId()?.to_string()?);
    let property_store = device.OpenPropertyStore(STGM_READ)?;
    let device_name =
        PropVariantToStringAlloc(&property_store.GetValue(&PKEY_Device_FriendlyName)?)?
            .to_string()?;
    println!("Device name: {}", device_name);
    return Ok(device);
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let audio_endpoint = unsafe {
        let microphone = get_microphone(cli.device_name).unwrap();
        let audio_endpoint = microphone.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
        Ok::<IAudioEndpointVolume, Error>(audio_endpoint)
    }
    .unwrap();

    let safe_audio_endpoint = SafeIAudioEndpointVolume {
        mutex: Mutex::new(WrappedIAudioEndpointVolumePointer(audio_endpoint)),
    };

    let mut hotkey = Hotkey::new();
    hotkey
        .register(
            Context::new()
                .modifiers(buttons!(LCtrl))
                .native_event_operation(NativeEventOperation::Block),
        )
        .on_press(Button::SideButton2, move |_| {
            let audio_endpoint = &safe_audio_endpoint.mutex.lock().unwrap().0;
            let local_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            unsafe {
                let muting_action = !audio_endpoint.GetMute().unwrap();
                let muting_message = match muting_action.into() {
                    true => "Muted",
                    false => "Unmuted",
                };
                audio_endpoint
                    .SetMute(!audio_endpoint.GetMute().unwrap(), ptr::null())
                    .unwrap();
                println!(
                    "{}",
                    format!("[{local_time}] Microphone is {muting_message}")
                );
            }
        });
    hotkey.install();

    // unsafe {
    //     CoInitializeEx(None, COINIT_MULTITHREADED)?;
    //     // let enumerator: IMMDeviceEnumerator =
    //     //     CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    //     // let default_endpoint = enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia)?;

    //     // let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
    //     // println!("Found devices: {}", collection.GetCount()?);
    //     // let device = collection.Item(0)?;
    //     // println!("Device id: {}", device.GetId()?.to_string()?);
    //     // println!(
    //     //     "Default audio endpoint id: {}",
    //     //     default_endpoint.GetId()?.to_string()?
    //     // );
    //     // let property_store = device.OpenPropertyStore(STGM_READ)?;
    //     // let device_name =
    //     //     PropVariantToStringAlloc(&property_store.GetValue(&PKEY_Device_FriendlyName)?)?
    //     //         .to_string()?;
    //     // println!("Device name: {}", device_name);
    //     // if device_name == "Microphone (4- G55)" {
    //     //     println!("Found micro!")
    //     // }

    //     // let audio_muter = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
    //     // println!(
    //     //     "Master volume level: {}",
    //     //     audio_muter.GetMasterVolumeLevel()?
    //     // );
    //     // println!(
    //     //     "Master volume scalar: {}",
    //     //     audio_muter.GetMasterVolumeLevelScalar()?
    //     // );
    //     // audio_muter.SetMasterVolumeLevelScalar(1.0, ptr::null());
    //     // audio_muter.SetMute(TRUE, ptr::null());
    //     // audio_muter.SetMute(FALSE, ptr::null());
    // }

    Ok(())
}
