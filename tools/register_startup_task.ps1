$trigger= New-ScheduledTaskTrigger -AtStartup
$user= "NT AUTHORITY\SYSTEM"
$action= New-ScheduledTaskAction -Execute "PowerShell.exe" -Argument "start-process -windowstyle hidden nog"

Register-ScheduledTask -TaskName "MonitorGroupMembership" -Trigger $trigger -User $user -Action $action -RunLevel Highest –Force
