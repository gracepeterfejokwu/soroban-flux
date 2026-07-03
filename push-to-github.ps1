# Secure GitHub Upload Script for Soroban Flux
# Requires: $env:GITHUB_PAT environment variable to be set with a valid GitHub Personal Access Token

param(
    [string]$Owner = "gracepeterfejokwu",
    [string]$Repo = "soroban-flux",
    [string]$Branch = "main"
)

# Configuration
$GitHubApiUrl = "https://api.github.com"
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$CommitMessage = "Initial commit: Production-ready Soroban Flux streaming protocol"
$ReleaseVersion = "v1.0.0"
$ReleaseNotes = @"
# Soroban Flux v1.0.0 - Production Release

Initial production-ready release of the Soroban Flux streaming protocol.

## Features
- Core flux engine smart contract
- Frontend visualization application
- Comprehensive documentation and guides
- Production deployment support
- Security and integration guidelines

## Repository
https://github.com/$Owner/$Repo
"@

# Helper function to show progress
function Write-Progress-Message {
    param([string]$Message, [string]$Type = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $colors = @{
        "INFO"    = "Cyan"
        "SUCCESS" = "Green"
        "ERROR"   = "Red"
        "WARN"    = "Yellow"
    }
    Write-Host "[$timestamp] [$Type] $Message" -ForegroundColor $colors[$Type]
}

# Validate GitHub PAT is set
function Test-GitHubAuth {
    if (-not $env:GITHUB_PAT) {
        Write-Progress-Message "ERROR: GITHUB_PAT environment variable not set" "ERROR"
        Write-Host ""
        Write-Host "To set the environment variable:"
        Write-Host "  `$env:GITHUB_PAT = 'your_github_pat_token_here'"
        Write-Host ""
        exit 1
    }
    
    # Verify the token works
    try {
        $headers = @{
            "Authorization" = "token $env:GITHUB_PAT"
            "Accept"        = "application/vnd.github.v3+json"
        }
        
        $response = Invoke-RestMethod -Uri "$GitHubApiUrl/user" -Headers $headers -Method Get -ErrorAction Stop
        Write-Progress-Message "GitHub authentication successful. Logged in as: $($response.login)" "SUCCESS"
        return $true
    }
    catch {
        Write-Progress-Message "GitHub authentication failed. Invalid PAT token." "ERROR"
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Get all files recursively, excluding .git and specified patterns
function Get-ProjectFiles {
    $ExcludePatterns = @(".git", ".gitkeep", "node_modules", ".next", "target", "dist")
    $files = @()
    
    Get-ChildItem -Path $ProjectRoot -Recurse -File | Where-Object {
        $fullPath = $_.FullName
        $excluded = $false
        
        foreach ($pattern in $ExcludePatterns) {
            if ($fullPath -like "*\$pattern\*" -or $fullPath -like "*\$pattern") {
                $excluded = $true
                break
            }
        }
        
        return -not $excluded
    } | ForEach-Object {
        $files += $_
    }
    
    return $files
}

# Encode file content (base64)
function Get-FileContent-Base64 {
    param([string]$FilePath)
    
    try {
        $fileBytes = [System.IO.File]::ReadAllBytes($FilePath)
        $base64 = [System.Convert]::ToBase64String($fileBytes)
        return $base64
    }
    catch {
        Write-Progress-Message "Failed to read file: $FilePath" "WARN"
        return $null
    }
}

# Determine if file is binary
function Test-BinaryFile {
    param([string]$FilePath)
    
    $binaryExtensions = @(".png", ".jpg", ".jpeg", ".gif", ".zip", ".exe", ".dll", ".so")
    $extension = [System.IO.Path]::GetExtension($FilePath).ToLower()
    
    return $binaryExtensions -contains $extension
}

# Upload file to GitHub
function Upload-File-ToGitHub {
    param(
        [string]$FilePath,
        [string]$RepositoryPath,
        [hashtable]$Headers
    )
    
    try {
        $base64Content = Get-FileContent-Base64 -FilePath $FilePath
        
        if ($null -eq $base64Content) {
            return $false
        }
        
        $uploadUrl = "$GitHubApiUrl/repos/$Owner/$Repo/contents/$RepositoryPath"
        
        $body = @{
            message = $CommitMessage
            content = $base64Content
            branch  = $Branch
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri $uploadUrl -Headers $Headers -Method Put -Body $body -ErrorAction Stop
        
        Write-Progress-Message "Uploaded: $RepositoryPath" "SUCCESS"
        return $true
    }
    catch {
        $errorMsg = $_.Exception.Response.Content | ConvertFrom-Json | Select-Object -ExpandProperty message
        Write-Progress-Message "Failed to upload $RepositoryPath : $errorMsg" "ERROR"
        return $false
    }
}

# Create release tag and release notes
function Create-GitHubRelease {
    param([hashtable]$Headers)
    
    try {
        Write-Progress-Message "Creating release tag: $ReleaseVersion" "INFO"
        
        # First, get the latest commit SHA
        $refUrl = "$GitHubApiUrl/repos/$Owner/$Repo/git/refs/heads/$Branch"
        $refResponse = Invoke-RestMethod -Uri $refUrl -Headers $Headers -Method Get -ErrorAction Stop
        $latestSha = $refResponse.object.sha
        
        # Create the release
        $releaseUrl = "$GitHubApiUrl/repos/$Owner/$Repo/releases"
        $releaseBody = @{
            tag_name = $ReleaseVersion
            target_commitish = $Branch
            name = $ReleaseVersion
            body = $ReleaseNotes
            draft = $false
            prerelease = $false
        } | ConvertTo-Json
        
        $releaseResponse = Invoke-RestMethod -Uri $releaseUrl -Headers $Headers -Method Post -Body $releaseBody -ErrorAction Stop
        
        Write-Progress-Message "Release created successfully: $($releaseResponse.html_url)" "SUCCESS"
        return $true
    }
    catch {
        $errorMsg = $_.Exception.Response.Content | ConvertFrom-Json | Select-Object -ExpandProperty message
        Write-Progress-Message "Failed to create release: $errorMsg" "WARN"
        return $false
    }
}

# Main execution
function Main {
    Write-Host ""
    Write-Progress-Message "Starting GitHub upload for $Owner/$Repo" "INFO"
    Write-Host ""
    
    # Validate authentication
    if (-not (Test-GitHubAuth)) {
        exit 1
    }
    
    Write-Host ""
    
    # Set up headers
    $headers = @{
        "Authorization" = "token $env:GITHUB_PAT"
        "Accept"        = "application/vnd.github.v3+json"
    }
    
    # Get all project files
    Write-Progress-Message "Scanning project files..." "INFO"
    $files = Get-ProjectFiles
    $totalFiles = $files.Count
    
    Write-Progress-Message "Found $totalFiles files to upload" "SUCCESS"
    Write-Host ""
    
    # Upload files
    $uploadedCount = 0
    $failedCount = 0
    
    foreach ($file in $files) {
        $relativePath = $file.FullName.Substring($ProjectRoot.Length + 1)
        $repositoryPath = $relativePath -replace "\\", "/"
        
        Write-Host "[$([int]($uploadedCount + $failedCount + 1))/$totalFiles] Uploading: $repositoryPath"
        
        if (Upload-File-ToGitHub -FilePath $file.FullName -RepositoryPath $repositoryPath -Headers $headers) {
            $uploadedCount++
        }
        else {
            $failedCount++
        }
        
        # Small delay to avoid API rate limiting
        Start-Sleep -Milliseconds 100
    }
    
    Write-Host ""
    Write-Progress-Message "Upload complete! Uploaded: $uploadedCount files" "SUCCESS"
    
    if ($failedCount -gt 0) {
        Write-Progress-Message "Failed uploads: $failedCount files" "WARN"
    }
    
    # Create release
    Write-Host ""
    if (Create-GitHubRelease -Headers $headers) {
        Write-Host ""
        Write-Progress-Message "✓ GitHub upload and release complete!" "SUCCESS"
        Write-Host ""
        Write-Host ("Repository URL: https://github.com/$Owner/$Repo") -ForegroundColor Green
        Write-Host ("Release URL:    https://github.com/$Owner/$Repo/releases/tag/$ReleaseVersion") -ForegroundColor Green
        Write-Host ""
    }
    else {
        Write-Host ""
        Write-Progress-Message "Files uploaded but release creation encountered issues. You can create it manually if needed." "WARN"
        Write-Host ""
        Write-Host ("Repository URL: https://github.com/" + $Owner + "/" + $Repo)
        Write-Host ""
    }
}

# Run main function
Main
