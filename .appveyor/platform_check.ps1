if ($env:platform.Contains("x86")) {
    if ($env:target.Contains("x86_64")) {
        Write-Host ("Skipping x64 buids on x86 platforms")
        exit -1
    }
}
