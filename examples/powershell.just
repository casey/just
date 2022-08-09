# Cross platform shebang:
shebang := if os() == 'windows' {
  'powershell.exe'
} else {
  '/usr/bin/env pwsh'
}

# Set shell for non-Windows OSs:
set shell := ["powershell", "-c"]

# Set shell for Windows OSs:
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# If you have PowerShell Core installed and want to use it,
# use `pwsh.exe` instead of `powershell.exe`

linewise:
  Write-Host "Hello, world!"

shebang:
	#!{{shebang}}
	$PSV = $PSVersionTable.PSVersion | % {"$_" -split "\." }
	$psver = $PSV[0] + "." + $PSV[1]
	if ($PSV[2].Length -lt 4) {
		$psver += "." + $PSV[2] + " Core"
	} else {
		$psver += " Desktop"
	}
	echo "PowerShell $psver"
