$url = "https://github.com/ctih1/kake/releases/download/latest/installer.exe"
$TEMP = [System.Environment]::GetEnvironmentVariable('TEMP','Machine')
$outpath = "$TEMP/inst.exe"
Invoke-WebRequest -Uri $url -OutFile $outpath
Remove-Item -Path $outpath