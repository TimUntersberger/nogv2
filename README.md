# nogv2

Complete rewrite of nog. This repo will get merged into the original nog repo once ready.

## Installation

For now you can only install `nog` by cloning the repository and doing one of the following:

### Using the Makefile

Executing `make` will create a new release with the version set to `CUSTOM`, install it 
and then start `nog` in the background.

### Using the tools

Execute the following powershell commands in the same order:

1. `.\tools\make_release.ps1 CUSTOM`
2. `.\tools\install_release.ps1 NogRelease.zip`
3. `start-process -windowstyle hidden nog`

## Structure

| project | description |
|-|-|
| nog-cli | Cli client for nog |
| nog-client | Client library for nog |
| nog-protocol | Defines a custom protocol for the cli client and nog to communicate with |
| nog-iced | A wrapper for iced winit which adds support for handling/modifying the created winit window |
| nog-bar | Hosts the code for the appbar |
| nog-menu | Hosts the code for the menu |
| nog-notif | An application that displays a notification style window based on the input arguments |
| nog | The tilingin window manager |

## Tools

The tools can be found in `./tools/*`.

| file | description |
|-|-|
| rcedit.exe | Utility tool for adding an icon to executables |
| sudo.ps1 | Runs a script in an elevated powershell prompt |
| make_release.ps1 <version> | Creates a NogRelease.zip file |
| install_release.ps1 <zip_file> | Moves the zip content into the install path and adds the install path to the path, if not already done. |
| register_startup_task.ps1 | Adds a scheduled task which runs at startup and starts nog (requires admin privileges) |
| unregister_startup_task.ps1 | Removes the scheduled task (requires admin privileges) |

## Usage

The `tools/install_release.ps1` script will add nog to the path. Afterwards you can start nog either
in the current shell

```powershell
nog
```

or inside a new hidden window

```powershell
start-process -windowstyle hidden nog
```

You can also use the `tools/register_startup_task.ps1` script to add nog to startup.

## Nightly

Nog requires nightly rust to build, because of the following reasons:

* We need the `raw_arg` method of `Command` so rust doesn't auto-escape our arguments. (https://github.com/rust-lang/rust/issues/29494)
