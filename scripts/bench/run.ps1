# Windows counterpart of run.sh. Startup measured separately and subtracted.
#   pwsh scripts/bench/run.ps1
$here = $PSScriptRoot
$root = Resolve-Path "$here\..\.."
$etamil = if ($env:ETAMIL_BIN) { $env:ETAMIL_BIN } else { "$root\etamil_compiler\target\release\etamil.exe" }
$runs = if ($env:RUNS) { [int]$env:RUNS } else { 7 }

if (-not (Test-Path $etamil)) {
    Write-Host "error: $etamil not found - (cd etamil_compiler; cargo build --release)"
    exit 1
}

function BestMs($block) {
    $t = @(); 1..$runs | ForEach-Object { $t += (Measure-Command $block).TotalMilliseconds }
    [math]::Round(($t | Measure-Object -Minimum).Minimum, 1)
}
function Row($a,$b,$c,$d) { "{0,-22} {1,9} {2,11} {3,9}" -f $a,$b,$c,$d }

Write-Host "=== answers (all must be 249997500) ==="
"  eTamil        $(& $etamil --vm "$here\tax.qmz" | Select-Object -Last 3 | Select-Object -First 1)"
if (Get-Command python -EA SilentlyContinue) { "  Python dec    $(python "$here\tax_decimal.py")" }
if (Get-Command python -EA SilentlyContinue) { "  Python float  $(python "$here\tax_float.py")" }
if (Get-Command node   -EA SilentlyContinue) { "  JavaScript    $(node "$here\tax.js")" }

Write-Host "`n=== timings, minimum of $runs runs ==="
Row "language" "total ms" "startup ms" "compute"

$eStart = BestMs { & $etamil --vm "$here\empty.qmz" | Out-Null }
$eTotal = BestMs { & $etamil --vm "$here\tax.qmz" | Out-Null }
Row "eTamil (decimal)" $eTotal $eStart ([math]::Round($eTotal - $eStart, 1))

if (Get-Command python -EA SilentlyContinue) {
    $pStart = BestMs { python "$here\empty.py" | Out-Null }
    foreach ($v in "decimal","float") {
        $t = BestMs { python "$here\tax_$v.py" | Out-Null }
        Row "Python ($v)" $t $pStart ([math]::Round($t - $pStart, 1))
    }
}
if (Get-Command node -EA SilentlyContinue) {
    $nStart = BestMs { node "$here\empty.js" | Out-Null }
    $t = BestMs { node "$here\tax.js" | Out-Null }
    Row "JavaScript (double)" $t $nStart ([math]::Round($t - $nStart, 1))
}

Write-Host "`n=== diagnostics: where the VM's time goes ==="
Row "program" "total ms" "startup ms" "compute"
foreach ($f in "loop_only","loop_mul","loop_vars","append") {
    $t = BestMs { & $etamil --vm "$here\$f.qmz" | Out-Null }
    Row $f $t $eStart ([math]::Round($t - $eStart, 1))
}

Write-Host "`nloop_only runs 900,000 bytecode instructions. compute_ms * 1e6 / 900000"
Write-Host "gives nanoseconds per instruction; a tuned bytecode VM sits at 5-15 ns."