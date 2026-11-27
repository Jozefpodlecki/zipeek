[CmdletBinding()]
param(
  [string]$OutFile     = ".\repo-context.md",
  [string]$LogFile     = ".\repo-context.log",
  [int]$MaxFileLines   = 600,
  [int]$MaxTotalLines  = 60000,
  [int]$MaxTreeDepth   = 4,
  [int]$HeartbeatEvery = 1,
  [int]$MaxFileSizeKB  = 512,
  [switch]$Quiet,
  [switch]$NoProgressBar
)

$ErrorActionPreference = 'Continue'
$root = (Get-Location).Path
$sw   = [Diagnostics.Stopwatch]::StartNew()

[Console]::WriteLine("")
[Console]::WriteLine("=====================================================")
[Console]::WriteLine(" capture-repo-context.ps1")
[Console]::WriteLine(" root   : $root")
[Console]::WriteLine(" out    : $OutFile")
[Console]::WriteLine(" log    : $LogFile")
[Console]::WriteLine(" cap    : $MaxTotalLines lines total, $MaxFileLines/file")
[Console]::WriteLine(" maxfile: $MaxFileSizeKB KB")
[Console]::WriteLine("=====================================================")
[Console]::WriteLine("")
[Console]::Out.Flush()

if (-not [IO.Path]::IsPathRooted($OutFile)) { $OutFile = Join-Path $root $OutFile }
if (-not [IO.Path]::IsPathRooted($LogFile)) { $LogFile = Join-Path $root $LogFile }

$excludeDirs = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@('node_modules','.git','target','dist','build','.next','.svelte-kit','bin','obj',
  '.venv','venv','__pycache__','.idea','.vscode','.cargo','coverage','.turbo',
  '.cache','pkg','out','gen','.tauri','.husky','.github',
  'external','vendor','third_party','third-party','thirdparty','assets','static',
  'public','data','fixtures','testdata','test-data','__fixtures__','migrations',
  'locales','i18n','translations','fonts','icons','images','img','media','audio',
  'video','samples','examples-data','models','archive-history',
  'dist-desktop','.desktop-assets','.stage') |
  ForEach-Object { [void]$excludeDirs.Add($_) }

$excludeExt = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@('.png','.jpg','.jpeg','.gif','.ico','.svg','.webp','.bmp','.woff','.woff2','.ttf',
  '.eot','.otf','.mp3','.mp4','.wav','.ogg','.webm','.zip','.gz','.tar','.7z','.rar',
  '.exe','.dll','.so','.dylib','.node','.wasm','.rlib','.rmeta','.map','.min.js',
  '.min.css','.pdf','.duckdb','.db') |
  ForEach-Object { [void]$excludeExt.Add($_) }

$binaryExt = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@('.png','.jpg','.jpeg','.gif','.ico','.webp','.woff','.woff2','.ttf','.exe','.dll',
  '.so','.dylib','.wasm','.rlib','.rmeta','.duckdb','.db') |
  ForEach-Object { [void]$binaryExt.Add($_) }

$sourceExt = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@('.rs','.toml','.yaml','.yml','.json','.json5','.html','.htm',
  '.css','.scss','.sass','.less','.js','.jsx','.ts','.tsx','.mjs','.cjs',
  '.vue','.svelte','.md','.mdx','.sh','.ps1','.bat','.cmd',
  '.sql','.graphql','.gql','.proto','.wgsl','.glsl','.vert','.frag') |
  ForEach-Object { [void]$sourceExt.Add($_) }

$knownNames = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@('Cargo.toml','Cargo.lock','rust-toolchain','rust-toolchain.toml','rustfmt.toml',
  'clippy.toml','package.json','pnpm-lock.yaml','yarn.lock','package-lock.json',
  'tauri.conf.json','tauri.conf.json5','Tauri.toml','Trunk.toml','index.html',
  'README','README.md','LICENSE','LICENSE.md','CONTRIBUTING.md','ARCHITECTURE.md',
  'DESIGN.md','Dockerfile','Makefile','justfile','build.rs','.gitignore',
  '.editorconfig','.rustfmt.toml','.clippy.toml','.env','.env.example','.env.sample',
  'vite.config.ts','vite.config.js','webpack.config.js','tsconfig.json','biome.json') |
  ForEach-Object { [void]$knownNames.Add($_) }

$skipNames = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
@([IO.Path]::GetFileName($OutFile), [IO.Path]::GetFileName($LogFile),
  'repo-context.md','repo-context.log','diagnose.ps1','capture-repo-context.ps1') |
  ForEach-Object { if ($_) { [void]$skipNames.Add($_) } }

$maxFileBytes = $MaxFileSizeKB * 1024

function Test-PathExcluded([string]$path) {
  $parts = $path.Replace('\','/').Split('/')
  foreach ($p in $parts) {
    if ($excludeDirs.Contains($p)) { return $true }
  }
  return $false
}

function Test-IsSourceFile([IO.FileInfo]$fi) {
  if ($knownNames.Contains($fi.Name)) { return $true }
  $ext = $fi.Extension
  if (-not $ext) { return $false }
  return $sourceExt.Contains($ext)
}

function Get-FilesFast {
  param(
    [string]$Root,
    [System.Collections.Generic.HashSet[string]]$ExcludeDirs,
    [System.Collections.Generic.HashSet[string]]$ExcludeExt,
    [long]$MaxBytes,
    [int]$HeartbeatEvery = 500,
    [scriptblock]$OnProgress
  )
  $results     = [System.Collections.Generic.List[IO.FileInfo]]::new()
  $stack       = [System.Collections.Generic.Stack[string]]::new()
  $stack.Push($Root)
  $visited     = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
  $scanned     = 0
  $dirsVisited = 0

  while ($stack.Count -gt 0) {
    $dir = $stack.Pop()
    if (-not $visited.Add($dir)) { continue }
    $dirsVisited++

    try {
      foreach ($f in [System.IO.Directory]::EnumerateFiles($dir)) {
        $scanned++
        try {
          $fi = [IO.FileInfo]::new($f)
          if ($fi.Length -ge $MaxBytes) { continue }
          $ext = $fi.Extension
          if ($ext -and $ExcludeExt.Contains($ext)) { continue }
          $results.Add($fi)
        } catch { }
        if ($HeartbeatEvery -gt 0 -and ($scanned % $HeartbeatEvery) -eq 0 -and $OnProgress) {
          & $OnProgress $scanned $results.Count $dirsVisited $dir
        }
      }
    } catch { }

    try {
      foreach ($d in [System.IO.Directory]::EnumerateDirectories($dir)) {
        $name = [IO.Path]::GetFileName($d)
        if ($ExcludeDirs.Contains($name)) { continue }
        try {
          $attr = [IO.File]::GetAttributes($d)
          if ($attr -band [IO.FileAttributes]::ReparsePoint) { continue }
        } catch { continue }
        $stack.Push($d)
      }
    } catch { }
  }

  return $results
}

$utf8 = [Text.UTF8Encoding]::new($false)
$md   = [IO.StreamWriter]::new($OutFile, $false, $utf8, 65536)
$log  = [IO.StreamWriter]::new($LogFile, $false, $utf8, 16384)
$md.AutoFlush  = $false
$log.AutoFlush = $false

function M([string]$s) { $md.WriteLine($s) }
function L([string]$s) {
  $line = "$(Get-Date -Format 'HH:mm:ss.fff')  $s"
  $log.WriteLine($line)
  if (-not $Quiet) { [Console]::WriteLine($line) }
}
function Phase([string]$s) {
  [Console]::WriteLine("")
  [Console]::WriteLine(">>> $s")
  [Console]::Out.Flush()
  $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  ### $s")
}

try {
  Phase "PHASE 1/3  enumerate files"
  $sw.Restart()

  $lastHb = [Diagnostics.Stopwatch]::StartNew()
  $onProgress = {
    param($scanned, $kept, $dirs, $currentDir)
    if ($lastHb.Elapsed.TotalSeconds -ge 1.0) {
      $rel = $currentDir
      if ($rel.Length -gt $root.Length) { $rel = $rel.Substring($root.Length).TrimStart('\') }
      [Console]::WriteLine("    ... scanned=$scanned  kept=$kept  dirs=$dirs  at: $rel")
      [Console]::Out.Flush()
      $lastHb.Restart()
    }
  }.GetNewClosure()

  $allFilesRaw = Get-FilesFast -Root $root `
    -ExcludeDirs $excludeDirs `
    -ExcludeExt  $excludeExt `
    -MaxBytes    $maxFileBytes `
    -HeartbeatEvery 500 `
    -OnProgress $onProgress

  $allFiles = [System.Collections.Generic.List[IO.FileInfo]]::new()
  $skippedNonSource = 0
  foreach ($fi in $allFilesRaw) {
    if ($skipNames.Contains($fi.Name)) { $skippedNonSource++; continue }
    if (Test-PathExcluded $fi.FullName) { $skippedNonSource++; continue }
    if (Test-IsSourceFile $fi) { $allFiles.Add($fi) } else { $skippedNonSource++ }
  }

  $dt = [math]::Round($sw.Elapsed.TotalSeconds, 2)
  [Console]::WriteLine("    -> $($allFiles.Count) source files (skipped $skippedNonSource non-source) in $dt s")
  $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  ENUM done: $($allFiles.Count) source, $skippedNonSource skipped, $dt s")

  Phase "PHASE 2/3  read source files (single pass)"
  $sw.Restart()
  $cache = [System.Collections.Generic.Dictionary[string,object]]::new([Math]::Max($allFiles.Count,1), [StringComparer]::Ordinal)
  $totalLines = 0; $totalChars = 0; $textCount = 0; $readErr = 0
  $i = 0; $total = $allFiles.Count
  foreach ($f in $allFiles) {
    $i++
    $rel = $f.FullName.Substring($root.Length).TrimStart('\')

    try {
      $t = [Diagnostics.Stopwatch]::StartNew()
      $c = [IO.File]::ReadAllText($f.FullName)
      $t.Stop()
      $lines = if ($c.Length -eq 0) { 0 } else { ($c -split "`n").Count }
      $cache[$f.FullName] = [pscustomobject]@{ Text = $c; Lines = $lines; Chars = $c.Length; ReadMs = $t.Elapsed.TotalMilliseconds }
      $totalLines += $lines; $totalChars += $c.Length; $textCount++

      if ($i % $HeartbeatEvery -eq 0 -and -not $Quiet) {
        [Console]::WriteLine(("    [{0,5}/{1}] {2,7} lines {3,10} B {4,6:0.0}ms  {5}" -f $i, $total, $lines, $c.Length, $t.Elapsed.TotalMilliseconds, $rel))
        [Console]::Out.Flush()
      }
      $log.WriteLine(("$(Get-Date -Format 'HH:mm:ss.fff')  READ {0,7} lines {1,10} B {2,6:0.0}ms  {3}" -f $lines, $c.Length, $t.Elapsed.TotalMilliseconds, $rel))
    } catch {
      $readErr++
      [Console]::WriteLine("    [ERR] $rel :: $($_.Exception.Message)")
      $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  READ-ERR $rel :: $($_.Exception.Message)")
    }

    if (-not $NoProgressBar -and $i % 10 -eq 0 -and $total -gt 0) {
      Write-Progress -Activity "Reading" -Status "$i / $total" -PercentComplete (100 * $i / $total)
    }
  }
  if (-not $NoProgressBar) { Write-Progress -Activity "Reading" -Completed }
  $dt = [math]::Round($sw.Elapsed.TotalSeconds, 2)
  [Console]::WriteLine("    -> $textCount files, $totalLines lines, $totalChars chars, $readErr errors, $dt s")
  [Console]::WriteLine("    -> est. tokens (chars/4): $([math]::Round($totalChars/4))")
  $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  READ done: $textCount files, $totalLines lines, $totalChars chars, $readErr errors, $dt s")

  Phase "PHASE 3/3  emit markdown"
  $sw.Restart()

  M "# Repo Context Capture"
  M ""
  M "Root: ``$root``"
  M "Captured: $(Get-Date -Format 'u')"
  M "PowerShell: $($PSVersionTable.PSVersion)"
  M ""

  M "## Git"
  $isGit = $false
  try { $isGit = (git rev-parse --is-inside-work-tree 2>$null) -eq 'true' } catch {}
  if ($isGit) {
    M "Branch: ``$(git rev-parse --abbrev-ref HEAD 2>$null)``"
    M "HEAD: ``$(git rev-parse --short HEAD 2>$null)``"
    M "Remote: ``$(git config --get remote.origin.url 2>$null)``"
    M ""
    M "### Status (short)"; M '```'; M ((git status --short 2>$null) -join "`n"); M '```'; M ""
    M "### Recent commits"; M '```'; M ((git log --oneline -n 20 2>$null) -join "`n"); M '```'; M ""
  } else {
    M "_Not a git repository or git not installed._"; M ""
  }

  M "## Size metrics"
  M "- Text files: $textCount"
  M "- Total lines: $totalLines"
  M "- Total chars: $totalChars"
  M "- Est. tokens (chars/4): $([math]::Round($totalChars/4))"
  M ""

  M "## LOC by extension (top 25)"
  M '```'
  $extGroups = $cache.GetEnumerator() |
    Group-Object { [IO.Path]::GetExtension($_.Key).ToLower() } |
    ForEach-Object {
      $sum = 0; foreach ($e in $_.Group) { $sum += $e.Value.Lines }
      [pscustomobject]@{ Ext = if ($_.Name) { $_.Name } else { '(none)' }; Files = $_.Count; Lines = $sum }
    } | Sort-Object Lines -Descending | Select-Object -First 25
  foreach ($g in $extGroups) { M ("{0,-12} {1,6} files  {2,10} lines" -f $g.Ext, $g.Files, $g.Lines) }
  M '```'; M ""

  M "## LOC by top-level directory"
  M '```'
  $dirGroups = @{}
  foreach ($kv in $cache.GetEnumerator()) {
    $rel = $kv.Key.Substring($root.Length).TrimStart('\')
    $parts = $rel -split '\\', 2
    $top = if ($parts.Count -gt 1) { $parts[0] } else { '(root)' }
    if (-not $dirGroups.ContainsKey($top)) { $dirGroups[$top] = [pscustomobject]@{ Files = 0; Lines = 0 } }
    $dirGroups[$top].Files++
    $dirGroups[$top].Lines += $kv.Value.Lines
  }
  foreach ($d in ($dirGroups.GetEnumerator() | Sort-Object { $_.Value.Lines } -Descending)) {
    M ("{0,-30} {1,6} files  {2,10} lines" -f $d.Key, $d.Value.Files, $d.Value.Lines)
  }
  M '```'; M ""

  M "## Directory tree (depth $MaxTreeDepth)"
  M '```'
  M "."
  function Show-Tree([string]$path, [int]$depth, [string]$prefix) {
    if ($depth -gt $MaxTreeDepth) { return }
    $items = @(Get-ChildItem -LiteralPath $path -Force -ErrorAction SilentlyContinue |
      Where-Object { -not $excludeDirs.Contains($_.Name) } |
      Sort-Object @{e={-$_.PSIsContainer}}, Name)
    for ($j = 0; $j -lt $items.Count; $j++) {
      $item = $items[$j]; $last = ($j -eq $items.Count - 1)
      $branch = if ($last) { '└── ' } else { '├── ' }
      $childPrefix = $prefix + $(if ($last) { '    ' } else { '│   ' })
      $suffix = if ($item.PSIsContainer) { '/' } else { '' }
      M "$prefix$branch$($item.Name)$suffix"
      if ($item.PSIsContainer) { Show-Tree $item.FullName ($depth + 1) $childPrefix }
    }
  }
  Show-Tree $root 1 ""
  M '```'; M ""

  $keyNames = @('README.md','Cargo.toml',
    'package.json','pnpm-lock.yaml','yarn.lock','package-lock.json',
    'tauri.conf.json','tauri.conf.json5','Tauri.toml','Trunk.toml','index.html',
    'rust-toolchain','rust-toolchain.toml','.rustfmt.toml','rustfmt.toml',
    'clippy.toml','.clippy.toml','CONTRIBUTING.md','ARCHITECTURE.md','DESIGN.md',
    'Dockerfile','docker-compose.yml','.env.example','.env.sample',
    'tsconfig.json','vite.config.ts','vite.config.js','webpack.config.js',
    '.gitignore','.editorconfig')
  $foundKey = [System.Collections.Generic.List[string]]::new()
  foreach ($name in $keyNames) { $p = Join-Path $root $name; if (Test-Path $p) { $foundKey.Add($p) } }
  Get-ChildItem -LiteralPath $root -Recurse -File -Force -ErrorAction SilentlyContinue -Include 'Cargo.toml','tauri.conf.json','tauri.conf.json5','Tauri.toml','Trunk.toml' |
    Where-Object { -not (Test-PathExcluded $_.FullName) } | ForEach-Object { $foundKey.Add($_.FullName) }

  M "## Key project files"
  $seen = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
  foreach ($p in $foundKey) {
    $resolved = (Resolve-Path -LiteralPath $p -ErrorAction SilentlyContinue).Path
    if (-not $resolved) { continue }
    if (-not $seen.Add($resolved)) { continue }
    $rel = $resolved.Substring($root.Length).TrimStart('\')
    M ""; M "### ``$rel``"; M '```'
    try {
      $content = [IO.File]::ReadAllText($resolved)
      $lines = if ($content.Length -eq 0) { 0 } else { ($content -split "`n").Count }
      if ($lines -gt $MaxFileLines) {
        M ((($content -split "`n") | Select-Object -First $MaxFileLines) -join "`n")
        M "... [truncated, file has $lines lines]"
      } else { M $content }
      L ("KEY  {0,7} lines  {1}" -f $lines, $rel)
    } catch { L "KEY-ERR $rel :: $($_.Exception.Message)" }
    M '```'
  }
  M ""

  M "## File inventory (grouped, first 200/group)"
  foreach ($group in ($cache.GetEnumerator() | Group-Object { [IO.Path]::GetExtension($_.Key).ToLower() } | Sort-Object Count -Descending)) {
    $ext = if ($group.Name) { $group.Name } else { '(no ext)' }
    M ""; M "### ``$ext`` ($($group.Count) files)"; M '```'
    $group.Group | Select-Object -First 200 | ForEach-Object { M $_.Key.Substring($root.Length).TrimStart('\') }
    M '```'
  }
  M ""

  M "## Dependency surface (Rust + JS)"
  $cargoRoot = Join-Path $root 'Cargo.toml'
  if (Test-Path $cargoRoot) {
    M ""; M "### Root Cargo.toml deps"; M '```'
    $cargo = Get-Content $cargoRoot -Raw
    $inDeps = $false
    foreach ($line in ($cargo -split "`n")) {
      if ($line -match '^\[.*dependencies.*\]') { $inDeps = $true; M $line; continue }
      if ($line -match '^\[') { $inDeps = $false }
      if ($inDeps) { M $line }
    }
    M '```'
  }
  $pkg = Join-Path $root 'package.json'
  if (Test-Path $pkg) {
    M ""; M "### package.json (deps sections)"; M '```json'
    try { $j = Get-Content $pkg -Raw | ConvertFrom-Json; M ($j | Select-Object name,version,dependencies,devDependencies | ConvertTo-Json -Depth 5) }
    catch { M "parse error" }
    M '```'
  }

  M ""; M "## Source files (Rust + Yew + Tauri)"
  $srcExt = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
  '.rs','.toml','.yaml','.yml','.json','.json5','.html','.htm',
  '.css','.scss','.sass','.less','.js','.jsx','.ts','.tsx','.mjs','.cjs',
  '.vue','.svelte','.sql','.graphql','.gql','.proto','.wgsl','.glsl','.vert','.frag' |
    ForEach-Object { [void]$srcExt.Add($_) }
  $skipLock = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
  'Cargo.lock','package-lock.json','pnpm-lock.yaml','yarn.lock' |
    ForEach-Object { [void]$skipLock.Add($_) }

  $includedLines = 0; $srcCount = 0
  $srcEntries = @($cache.GetEnumerator() |
    Where-Object { $srcExt.Contains([IO.Path]::GetExtension($_.Key)) -and -not $skipLock.Contains([IO.Path]::GetFileName($_.Key)) } |
    Sort-Object @{
      e = {
        $r = $_.Key
        if ($r -match '\\src-tauri\\') { 0 }
        elseif ($r -match '\\crates\\[^\\]+\\src\\' -and $r -like '*.rs') { 1 }
        elseif ($r -like '*.rs') { 2 }
        elseif ($r -like '*.toml') { 3 }
        else { 9 }
      }
    }, Key)
  $totalSrc = $srcEntries.Count; $k = 0
  foreach ($entry in $srcEntries) {
    $k++
    if ($includedLines -ge $MaxTotalLines) {
      M ""; M "> Reached MaxTotalLines ($MaxTotalLines). Remaining source omitted."
      L "SRC  cap reached at $k/$totalSrc"
      break
    }
    $rel = $entry.Key.Substring($root.Length).TrimStart('\')
    $content = $entry.Value.Text
    $lines = $entry.Value.Lines
    M ""; M "### ``$rel``"; M '```rust'
    if ($lines -gt $MaxFileLines) {
      M ((($content -split "`n") | Select-Object -First $MaxFileLines) -join "`n")
      M "... [truncated: $lines total lines]"
    } else { M $content }
    M '```'
    $includedLines += [math]::Min($lines, $MaxFileLines)
    $srcCount++
    if (-not $Quiet) {
      [Console]::WriteLine(("    SRC [{0}/{1}] {2,7} lines  {3}" -f $k, $totalSrc, $lines, $rel))
      [Console]::Out.Flush()
    }
    $log.WriteLine(("$(Get-Date -Format 'HH:mm:ss.fff')  SRC  {0,7} lines  {1}" -f $lines, $rel))
  }
  $dt = [math]::Round($sw.Elapsed.TotalSeconds, 2)
  [Console]::WriteLine("    -> wrote $srcCount/$totalSrc source files, $includedLines lines included, $dt s")
  $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  EMIT done: $srcCount files, $includedLines lines, $dt s")
}
catch {
  [Console]::WriteLine("FATAL: $($_.Exception.Message)")
  [Console]::WriteLine($_.ScriptStackTrace)
  if ($log) { $log.WriteLine("$(Get-Date -Format 'HH:mm:ss.fff')  FATAL $($_.Exception.Message)"); $log.WriteLine($_.ScriptStackTrace) }
  throw
}
finally {
  if ($md)  { $md.Flush();  $md.Dispose() }
  if ($log) {
    $log.WriteLine("=== TOTAL $([math]::Round($sw.Elapsed.TotalSeconds,2))s ===")
    $log.Flush(); $log.Dispose()
  }

  [Console]::WriteLine("")
  [Console]::WriteLine("=====================================================")
  [Console]::WriteLine(" DONE")
  if (Test-Path $OutFile) {
    [Console]::WriteLine("   md  : $OutFile ($([math]::Round((Get-Item $OutFile).Length/1KB,1)) KB)")
  }
  [Console]::WriteLine("   log : $LogFile")
  [Console]::WriteLine("   time: $([math]::Round($sw.Elapsed.TotalSeconds,2))s")
  [Console]::WriteLine("=====================================================")
  [Console]::Out.Flush()
}