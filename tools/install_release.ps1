param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $file
)

$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA\nog"

expand-archive $file $asset_name

if (test-path $out_path) {
  remove-item -path "$out_path/runtime" -Recurse
  remove-item -path "$out_path/bin" -Recurse

  move-item "$asset_name/NogRelease/runtime" "$out_path/runtime"
  move-item "$asset_name/NogRelease/bin" "$out_path/bin"
} else {
  move-item "$asset_name/NogRelease" $out_path
}

remove-item $file
remove-item "./$asset_name" -Recurse

$nog_path = "$out_path\bin"
$old_path = [Environment]::GetEnvironmentVariable("PATH", "User")
$path_items = $old_path.split(";")
$path_has_nog = $path_items -contains $nog_path

echo "Adding nog to path..."
if (!$path_has_nog) {
  $path_items += $nog_path
  $new_path = $path_items -join ";"
  [Environment]::SetEnvironmentVariable('PATH', $new_path, 'User')
  # Start-Process powershell -Verb runAs -ArgumentList "[Environment]::SetEnvironmentVariable('PATH', '$new_path', 'User')"
  echo "Added nog to path."
} else {
  echo "Nog is already in the path."
}
