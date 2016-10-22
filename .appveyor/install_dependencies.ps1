# Current sqlite version in nuget repository.
$sqlite_version = "3.8.8.3"

# Download the latest nuget
appveyor DownloadFile https://dist.nuget.org/win-x86-commandline/latest/nuget.exe
# Use it to install sqlite3
.\nuget install sqlite3 -version $sqlite_version

if ($env:target.Contains("i686")) {
    $platform_path = "\Win32\v120"
} else {
    $platform_path = "\x64\v120"
}

if ($env:CONFIGURATION.Contains("release")) {
    $build_type_path = "\Release"
} elseif ($env:CONFIGURATION.Contains("debug")) {
    $build_type_path = "\Debug"
}

# Set the proper environment variables for the build
# AppVeyor env variables: https://www.appveyor.com/docs/environment-variables/
$sqlite3_lib = $env:APPVEYOR_BUILD_FOLDER + "\sqlite3." + $sqlite_version + "\build\native\lib"

# Put it all together and set the environment variable
$SQLITE3_LIB_DIR = $sqlite3_lib + $platform_path + $build_type_path
[Environment]::SetEnvironmentVariable("SQLITE3_LIB_DIR", $SQLITE3_LIB_DIR, "Process")
