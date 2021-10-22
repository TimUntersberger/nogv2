install-local:
	pwsh -c .\tools\make_release.ps1 CUSTOM;.\tools\install_release.ps1 NogRelease.zip
