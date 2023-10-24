set shell := ["powershell.exe", "-c"]

run:
    cargo run


run_with_device:
    cargo run -- --device-name "Microphone (4- G55)"