#![windows_subsystem = "windows"]
#![allow(unused)]
use rodio::Sink;
use std::io::Cursor;
use windows::core::HSTRING;

use std::ptr::{self};

use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_DEFBUTTON1, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};
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
use hookmap::prelude::*;
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::SyncSender;

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
    KeybindPressed,
    MicrophoneMuted,
    MicrophoneUnmuted,
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
        return Err(Error::from_win32());
    } else {
        return Ok(enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia)?);
    };
}

unsafe fn swap_microphone_muting_state(
    audio_endpoint: &IAudioEndpointVolume,
    sender: &SyncSender<Message>,
) {
    unsafe {
        let muting_action = !audio_endpoint.GetMute().unwrap();
        match muting_action.into() {
            true => sender.send(Message::MicrophoneMuted),
            false => sender.send(Message::MicrophoneUnmuted),
        };
        audio_endpoint.SetMute(muting_action, ptr::null()).unwrap();
    }
}

unsafe fn get_audio_endpoint(
    searched_device_name: Option<String>,
) -> Result<(IAudioEndpointVolume, String), Error> {
    unsafe {
        let microphone = match get_microphone(searched_device_name) {
            Ok(microphone) => microphone,
            Err(e) => {
                MessageBoxW(
                    None,
                    &HSTRING::from("Could not find working microphone."),
                    &HSTRING::from("Error"),
                    MB_ICONERROR | MB_OK | MB_DEFBUTTON1,
                );
                return Err(e);
            }
        };
        let device_name = get_device_name(&microphone).unwrap();
        MessageBoxW(
            None,
            &HSTRING::from(format!("Found microphone: {}", device_name)),
            &HSTRING::from("Success"),
            MB_ICONINFORMATION | MB_OK | MB_DEFBUTTON1,
        );
        let audio_endpoint = microphone.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
        Ok::<(IAudioEndpointVolume, String), Error>((audio_endpoint, device_name))
    }
}

fn init_muting_hotkey(sender: SyncSender<Message>) {
    thread::spawn(move || {
        let mut hotkey = Hotkey::new();
        hotkey
            .register(
                Context::new()
                    .modifiers(buttons!(LAlt))
                    .native_event_operation(NativeEventOperation::Block),
            )
            .on_press(Button::SideButton2, move |_| {
                sender.send(Message::KeybindPressed);
            });
        hotkey.install();
    });
}
// TODO: Select microphone through UI.
// TODO: Allow selecting keybind of your own
// TODO: Allow user select volume somehow..

fn main() -> Result<(), Error> {
    let mut tray = TrayItem::new("Microphone Muter", IconSource::Resource("aa-exe-icon")).unwrap();

    let (audio_endpoint, device_name) =
        unsafe { get_audio_endpoint(Cli::parse().device_name) }.unwrap();
    tray.add_label(format!("Selected device: {device_name}").as_str());

    let (tx, rx) = mpsc::sync_channel(2);

    init_muting_hotkey(tx.clone());

    let quit_sender = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_sender.send(Message::Quit);
    });

    let microphone_state_sender = tx.clone();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.set_volume(0.2);
    let activated_sound = Cursor::new(include_bytes!("../sounds/Activated.wav"));
    let muted_sound = Cursor::new(include_bytes!("../sounds/Muted.wav"));

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                tray.inner_mut().quit();
                tray.inner_mut().shutdown().unwrap();
                process::exit(0);
            }
            Ok(Message::KeybindPressed) => {
                unsafe { swap_microphone_muting_state(&audio_endpoint, &microphone_state_sender) };
            }
            Ok(Message::MicrophoneMuted) => {
                sink.skip_one();
                sink.append(Decoder::new(muted_sound.clone()).unwrap());
            }
            Ok(Message::MicrophoneUnmuted) => {
                sink.skip_one();
                sink.append(Decoder::new(activated_sound.clone()).unwrap());
            }
            _ => {}
        }
    }
}
