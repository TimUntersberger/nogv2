param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $Version
)

$root_dir = "NogRelease"

$env:NOG_VERSION=$Version

cargo build --release

if (!$?) {
    echo "Build was not successful. Aborting."
    return
}

if (test-path ./$root_dir.zip) {
    remove-item -Path ./$root_dir.zip
}

new-item -path . -name $root_dir -itemtype "Directory"
new-item -path ./$root_dir -name "runtime" -itemtype "Directory"
new-item -path ./$root_dir -name "config" -itemtype "Directory"
new-item -path ./$root_dir -name "bin" -itemtype "Directory"

copy-item ./nog/runtime/* ./$root_dir/runtime -recurse
copy-item ./target/release/nog.exe ./$root_dir/bin/nog.exe
copy-item ./target/release/nog-cli.exe ./$root_dir/bin/nog-cli.exe
copy-item ./target/release/nog-menu.exe ./$root_dir/bin/nog-menu.exe
copy-item ./target/release/nog-notif.exe ./$root_dir/bin/nog-notif.exe
copy-item ./target/release/nog-bar.exe ./$root_dir/bin/nog-bar.exe

compress-archive ./$root_dir ./$root_dir.zip

remove-item -Path ./$root_dir -recurse
