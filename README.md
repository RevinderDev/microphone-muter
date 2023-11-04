# Microphone Muter

Mute your microphone globally on Windows with just one click. Unwanted solution for a problem that no one had.

## Why?

Because different apps such as slack, discord, zoom, google meetings, games, steam, battle net, teamspeak and ventrillo (yes, sometimes) support different keybinds for muting your microphone. Then there are also fun ones which allow call host to force unmute you - looking at you `****`..

## Have you heard of hardware switches?

Yes and mine broke, therefore this solution. 


## Ah but you can do it in 2 lines of autohotkey.

[Yes, that's true](https://www.autohotkey.com/docs/v2/lib/SoundSetMute.htm) and here is a snippet for that:

```ahk
!XButton2::  ; Win + Z

SoundSet, +1, MASTER, mute, 6 ; 6 is my microphone ID, edit in your own.
SoundGet, master_mute, , mute, 6

ToolTip, Mute %master_mute% ;use a tool tip at mouse pointer to show what state mic is after toggle
SetTimer, RemoveToolTip, -1000
return

RemoveToolTip:
ToolTip
return
```

But seeing uproar in gaming against AHK users as bannable offense (rightfully so) I decided to write my own solution.

# How to

Double click and voila. You can now use fantastic personal keybind `LAlt + MouseButton4` to mute yourself.

If you wanna specify different than default device then you can do it using it's name from terminal like so:

```sh
$ microphone-muter --device-name "Microphone (4- G55)"    
```


## Development

You can run it with cargo but for the sake of easiness I added [just](https://github.com/casey/just).

Building
```sh
$ cargo build
$ cargo build --release
```

Running
```sh
$ cargo run
$ just
$ just run_with_device
```

TODO:
- [x] Add support for different devices
- [x] Add some representation to current state of microphone (in form of sound or popup)
- [ ] Support linux
- [ ] Support letting user choose device in the interface itself
- [ ] Maybe add actual interface than this crap
- [ ] Allow user to change volume in said interface
- [ ] Add CI/CD
- [ ] Add tests (yay..)
- [ ] Add your own icon and sound indicators