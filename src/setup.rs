use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
const STATUSLINE_SCRIPT: &str = r#"#!/bin/bash
# abtop StatusLine hook — writes rate limit data for abtop to read.
# Installed by: abtop --setup
# Reads JSON from stdin with a 5s timeout, pipes it to python via stdin
# to avoid ARG_MAX limits on large payloads.
INPUT=""
while IFS= read -r -t 5 line || [ -n "$line" ]; do
    INPUT="${INPUT}${line}
"
done
[ -z "$INPUT" ] && exit 0
printf '%s' "$INPUT" | python3 -c "
import sys, json, time, os
data = json.load(sys.stdin)
rl = data.get('rate_limits')
if not rl:
    sys.exit(0)
out = {'source': 'claude', 'updated_at': int(time.time())}
fh = rl.get('five_hour')
if fh:
    out['five_hour'] = {'used_percentage': fh.get('used_percentage', 0), 'resets_at': fh.get('resets_at', 0)}
sd = rl.get('seven_day')
if sd:
    out['seven_day'] = {'used_percentage': sd.get('used_percentage', 0), 'resets_at': sd.get('resets_at', 0)}
config_dir = os.environ.get('CLAUDE_CONFIG_DIR', os.path.join(os.path.expanduser('~'), '.claude'))
with open(os.path.join(config_dir, 'abtop-rate-limits.json'), 'w') as f:
    json.dump(out, f)
" 2>/dev/null
"#;

#[cfg(target_os = "windows")]
const STATUSLINE_SCRIPT: &str = r#"# abtop StatusLine hook - writes rate limit data for abtop to read.
# Installed by: abtop --setup
$ErrorActionPreference = "SilentlyContinue"
$inputJson = [Console]::In.ReadToEnd()
if ([string]::IsNullOrWhiteSpace($inputJson)) { exit 0 }

$data = $inputJson | ConvertFrom-Json
if ($null -eq $data.rate_limits) { exit 0 }

$out = [ordered]@{
  source = "claude"
  updated_at = [int][DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
}

if ($null -ne $data.rate_limits.five_hour) {
  $out.five_hour = [ordered]@{
    used_percentage = $data.rate_limits.five_hour.used_percentage
    resets_at = $data.rate_limits.five_hour.resets_at
  }
}
if ($null -ne $data.rate_limits.seven_day) {
  $out.seven_day = [ordered]@{
    used_percentage = $data.rate_limits.seven_day.used_percentage
    resets_at = $data.rate_limits.seven_day.resets_at
  }
}

$configDir = $env:CLAUDE_CONFIG_DIR
if ([string]::IsNullOrWhiteSpace($configDir)) {
  $configDir = Join-Path $HOME ".claude"
}
$out | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $configDir "abtop-rate-limits.json") -Encoding UTF8
"#;

fn claude_dir() -> PathBuf {
    std::env::var("CLAUDE_CONFIG_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".claude"))
}

fn script_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        claude_dir().join("abtop-statusline.ps1")
    }
    #[cfg(not(target_os = "windows"))]
    {
        claude_dir().join("abtop-statusline.sh")
    }
}

fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

pub fn run_setup() {
    println!("abtop --setup: configuring Claude Code StatusLine hook\n");

    // Ensure ~/.claude directory exists
    let dir = claude_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("  ✗ failed to create {}: {}", dir.display(), e);
        std::process::exit(1);
    }

    // Step 1: Write the statusline script
    let script = script_path();
    match fs::write(&script, STATUSLINE_SCRIPT) {
        Ok(_) => {
            // chmod +x
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&script, fs::Permissions::from_mode(0o700));
            }
            println!("  ✓ wrote {}", script.display());
        }
        Err(e) => {
            eprintln!("  ✗ failed to write {}: {}", script.display(), e);
            std::process::exit(1);
        }
    }

    // Step 2: Update settings.json
    let settings_file = settings_path();
    let mut settings: Value = if settings_file.exists() {
        let content = match fs::read_to_string(&settings_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ✗ cannot read {}: {}", settings_file.display(), e);
                std::process::exit(1);
            }
        };
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "  ✗ {} contains invalid JSON: {}",
                    settings_file.display(),
                    e
                );
                eprintln!("    fix the file manually before running --setup");
                std::process::exit(1);
            }
        }
    } else {
        Value::Object(Default::default())
    };

    let obj = settings.as_object_mut().unwrap();
    let expected_cmd = statusline_command(&script);

    // Check if statusLine is already configured
    if let Some(existing) = obj.get("statusLine") {
        if let Some(existing_obj) = existing.as_object() {
            if let Some(cmd) = existing_obj.get("command") {
                let cmd_str = cmd.as_str().unwrap_or("");
                if cmd_str != expected_cmd && !cmd_str.is_empty() {
                    eprintln!("  ⚠ statusLine already configured: {}", cmd_str);
                    eprintln!("    to override, remove the existing statusLine key from:");
                    eprintln!("    {}", settings_file.display());
                    std::process::exit(1);
                }
            }
        }
    }

    // Set statusLine config
    obj.insert(
        "statusLine".to_string(),
        serde_json::json!({
            "type": "command",
            "command": expected_cmd
        }),
    );

    match fs::write(
        &settings_file,
        serde_json::to_string_pretty(&settings).unwrap_or_default(),
    ) {
        Ok(_) => println!("  ✓ updated {}", settings_file.display()),
        Err(e) => {
            eprintln!("  ✗ failed to update {}: {}", settings_file.display(), e);
            std::process::exit(1);
        }
    }

    println!("\n  done! rate limit data will appear in abtop after the next Claude response.");
    println!("  restart any running Claude Code sessions to activate.");
}

#[cfg(target_os = "windows")]
fn statusline_command(script: &std::path::Path) -> String {
    format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        script.display()
    )
}

#[cfg(not(target_os = "windows"))]
fn statusline_command(script: &std::path::Path) -> String {
    script.display().to_string()
}
