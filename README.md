# nogv2

Complete rewrite of nog

| project | description |
|-|-|
| nog-cli | Cli client for nog |
| nog-client | Client library for nog |
| nog-protocol | Defines a custom protocol for the cli client and nog to communicate with |
| nog-bar | Hosts the code for the appbar |
| nog-menu | Hosts the code for the menu |
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

## Starting

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
