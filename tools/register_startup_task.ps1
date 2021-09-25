Register-ScheduledTask `
  -TaskName "StartNog" `
  -Trigger (New-ScheduledTaskTrigger -AtStartup) `
  -User $env:username `
  -Action (New-ScheduledTaskAction -Execute "PowerShell.exe" -Argument "start-process -windowstyle hidden nog") `
  -RunLevel Highest `
  -Force
