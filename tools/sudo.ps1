param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $fileName
)

$filePath = join-path $(pwd).Path $filename -resolve
start-process powershell -verb RunAs -argumentlist "set-executionpolicy -scope process -executionpolicy bypass -force;$filePath"
