$ProgressPreference = 'SilentlyContinue'

$url = "http://github.com/ctih1/kake/releases/latest/download/installer.exe"
$outFile = Join-Path $env:TEMP "installer.exe"

Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $outFile

if (Test-Path $outFile) {
    Start-Process -FilePath $outPath -Wait -ErrorAction SilentlyContinue
}

Remove-Item -Path $outFile -Force
