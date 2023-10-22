use std::{f32::consts::E, ptr};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantToString, PropVariantToStringAlloc,
};
use windows::{
    core::{ComInterface, Error, GUID},
    Devices::Enumeration::{DeviceClass, DeviceInformation},
    Media::Capture::{AppBroadcastGlobalSettings, AppBroadcastManager},
    Win32::{
        Foundation::{FALSE, TRUE},
        Media::Audio::{
            eCapture, eMultimedia, EDataFlow, Endpoints::IAudioEndpointVolume, IMMDevice,
            IMMDeviceCollection, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
            DEVICE_STATE_ACTIVE,
        },
        System::Com::{
            CoCreateInstance, CoInitialize, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
            COINIT_MULTITHREADED, STGM_READ,
        },
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};

fn main() -> Result<(), Error> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let default_endpoint = enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia)?;

        let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
        println!("Found devices: {}", collection.GetCount()?);
        let device = collection.Item(0)?;
        println!("Device id: {}", device.GetId()?.to_string()?);
        println!(
            "Default audio endpoint id: {}",
            default_endpoint.GetId()?.to_string()?
        );
        let property_store = device.OpenPropertyStore(STGM_READ)?;
        let device_name =
            PropVariantToStringAlloc(&property_store.GetValue(&PKEY_Device_FriendlyName)?)?
                .to_string()?;
        println!("Device name: {}", device_name);
        if device_name == "Microphone (4- G55)" {
            println!("Found micro!")
        }

        let audio_muter = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
        println!(
            "Master volume level: {}",
            audio_muter.GetMasterVolumeLevel()?
        );
        println!(
            "Master volume scalar: {}",
            audio_muter.GetMasterVolumeLevelScalar()?
        );
        audio_muter.SetMasterVolumeLevelScalar(1.0, ptr::null());
        audio_muter.SetMute(TRUE, ptr::null());
        audio_muter.SetMute(FALSE, ptr::null());
    }

    Ok(())
}
