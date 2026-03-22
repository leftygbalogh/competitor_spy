# setup_google_places.ps1
# Interactively sets up the Google Places API key for competitor-spy,
# then runs a quick live check to confirm the key works.
#
# Usage:
#   .\scripts\setup_google_places.ps1
#
# What it does:
#   1. Asks for your Google Places API key (input hidden)
#   2. Asks for a credential store passphrase (input hidden)
#   3. Stores the key encrypted in %APPDATA%\competitor-spy\credentials
#   4. Lists stored credentials to confirm
#   5. Asks for a test industry + location and runs a live search

param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Binary = ".\target\release\competitor-spy.exe"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Read-HiddenInput {
    param([string]$Prompt)
    Write-Host -NoNewline "$Prompt"
    $secure = Read-Host -AsSecureString
    $bstr   = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try {
        return [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
    } finally {
        [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
    }
}

function Write-Step {
    param([string]$Text)
    Write-Host ""
    Write-Host "  >> $Text" -ForegroundColor Cyan
}

function Write-OK {
    param([string]$Text)
    Write-Host "  [OK] $Text" -ForegroundColor Green
}

function Write-Fail {
    param([string]$Text)
    Write-Host "  [FAIL] $Text" -ForegroundColor Red
}

# ---------------------------------------------------------------------------
# Pre-flight: binary present?
# ---------------------------------------------------------------------------

Write-Host ""
Write-Host "================================================" -ForegroundColor Yellow
Write-Host "  competitor-spy  --  Google Places setup" -ForegroundColor Yellow
Write-Host "================================================" -ForegroundColor Yellow

if (-not (Test-Path $Binary)) {
    Write-Fail "Binary not found at $Binary"
    Write-Host "  Build it first: cargo build --release -p competitor_spy_cli"
    exit 1
}
Write-OK "Binary found: $Binary"

# ---------------------------------------------------------------------------
# Step 1: Collect inputs
# ---------------------------------------------------------------------------

Write-Step "Enter your Google Places API key"
Write-Host "  (Get one at: https://console.cloud.google.com -> APIs & Services -> Credentials)" -ForegroundColor DarkGray
Write-Host "  Make sure 'Places API (New)' is enabled in your project." -ForegroundColor DarkGray
$ApiKey = Read-HiddenInput "  API key: "

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
    Write-Fail "API key cannot be empty."
    exit 1
}

Write-Host ""
Write-Step "Choose a passphrase for the encrypted credential store"
Write-Host "  This passphrase protects the stored key on disk. You'll need it every time" -ForegroundColor DarkGray
Write-Host "  you run competitor-spy in a new shell (set CSPY_CREDENTIAL_PASSPHRASE)." -ForegroundColor DarkGray
$Pass1 = Read-HiddenInput "  Passphrase: "
$Pass2 = Read-HiddenInput "  Confirm passphrase: "

if ($Pass1 -ne $Pass2) {
    Write-Fail "Passphrases do not match."
    exit 1
}
if ([string]::IsNullOrWhiteSpace($Pass1)) {
    Write-Fail "Passphrase cannot be empty."
    exit 1
}

# ---------------------------------------------------------------------------
# Step 2: Store the key
# ---------------------------------------------------------------------------

Write-Host ""
Write-Step "Storing key in encrypted credential store..."

$env:CSPY_CREDENTIAL_PASSPHRASE = $Pass1

try {
    $ApiKey | & $Binary credentials set google_places 2>&1 | ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Failed to store credential (exit $LASTEXITCODE)."
        exit 1
    }
    Write-OK "Key stored."
} catch {
    Write-Fail "Unexpected error: $_"
    exit 1
}

# ---------------------------------------------------------------------------
# Step 3: Verify list
# ---------------------------------------------------------------------------

Write-Step "Verifying stored credentials..."
$listOut = & $Binary credentials list 2>&1
$listOut | ForEach-Object { Write-Host "  $_" }
if ($LASTEXITCODE -ne 0) {
    Write-Fail "credentials list returned exit $LASTEXITCODE."
    exit 1
}

if ($listOut -notmatch "google_places\s+SET") {
    Write-Fail "google_places does not show as SET. Something went wrong."
    exit 1
}
Write-OK "google_places is SET."

# ---------------------------------------------------------------------------
# Step 4: Live search test
# ---------------------------------------------------------------------------

Write-Host ""
Write-Step "Live search test"
Write-Host "  We'll run a quick search to confirm the key works end-to-end." -ForegroundColor DarkGray

$Industry = Read-Host "  Industry to search for (e.g. cafe, gym, pilates)"
$Location = Read-Host "  Location (e.g. Vienna, Austria)"

if ([string]::IsNullOrWhiteSpace($Industry)) { $Industry = "cafe" }
if ([string]::IsNullOrWhiteSpace($Location)) { $Location = "Vienna, Austria" }

Write-Host ""
Write-Host "  Running: competitor-spy --industry `"$Industry`" --location `"$Location`" --radius 5 --no-pdf" -ForegroundColor DarkGray
Write-Host ""

& $Binary --industry $Industry --location $Location --radius 5 --no-pdf 2>&1

$exitCode = $LASTEXITCODE

Write-Host ""
if ($exitCode -eq 0) {
    Write-OK "Search completed (exit 0)."
} else {
    Write-Host "  [WARN] Search exited with code $exitCode. Check output above for details." -ForegroundColor Yellow
    Write-Host "  A non-zero exit may just mean geocoding failed or Google returned no results." -ForegroundColor DarkGray
    Write-Host "  The key is still stored — try a different location." -ForegroundColor DarkGray
}

# ---------------------------------------------------------------------------
# Step 5: Remind user how to use in future sessions
# ---------------------------------------------------------------------------

Write-Host ""
Write-Host "================================================" -ForegroundColor Yellow
Write-Host "  Setup complete!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "  In every new PowerShell session, set your passphrase before running searches:" -ForegroundColor White
Write-Host ""
Write-Host '  $env:CSPY_CREDENTIAL_PASSPHRASE = "<your-passphrase>"' -ForegroundColor Cyan
Write-Host '  .\target\release\competitor-spy.exe --industry "cafe" --location "Vienna, Austria" --radius 5' -ForegroundColor Cyan
Write-Host ""
Write-Host "  To delete the stored key later:" -ForegroundColor DarkGray
Write-Host '  .\target\release\competitor-spy.exe credentials delete google_places' -ForegroundColor DarkGray
Write-Host ""
