@echo off
cargo build --release
mkdir PhotoSelector
copy target\release\photo-selector.exe PhotoSelector\PhotoSelector.exe
powershell Compress-Archive -Path PhotoSelector -DestinationPath PhotoSelector-Windows.zip 