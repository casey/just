# This script takes care of packaging the build artifacts that will go in the
# release zipfile

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"

# DONE Update this to package the right artifacts
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\just.exe" '.\'
Copy-Item "$SRC_DIR\GRAMMAR.md" '.\'
Copy-Item "$SRC_DIR\LICENSE.md" '.\'
Copy-Item "$SRC_DIR\README.md" '.\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR
