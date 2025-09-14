$url = "http://github.com/ctih1/kake/releases/latest/download/installer.exe"
$TEMP = [System.Environment]::GetEnvironmentVariable('TEMP','Machine')
$outpath = "$TEMP/inst.exe"
Invoke-WebRequest -Uri $url -OutFile $outpath
Remove-Item -Path $outpath