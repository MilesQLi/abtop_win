# abtop for Windows

Windows-native build of [abtop](https://github.com/graykode/abtop), an AI agent monitor for the terminal.

The original project is like `btop`, but for Claude Code, Codex CLI, and OpenCode sessions. This fork exists to make it usable directly on Windows, without WSL or Linux/macOS-only process tools.

![abtop demo](https://raw.githubusercontent.com/graykode/abtop/main/assets/demo.gif)

Screenshot/demo credit: upstream [graykode/abtop](https://github.com/graykode/abtop).

## Download

Download the latest Windows binary from:

[GitHub Releases](https://github.com/MilesQLi/abtop_win/releases/latest)

Recommended asset:

- `abtop-windows-x86_64.zip` - includes `abtop.exe`, `LICENSE`, and `README.md`

You can also download `abtop.exe` directly.

## Usage

Open PowerShell or Windows Terminal in the folder containing `abtop.exe`:

```powershell
.\abtop.exe
```

Useful commands:

```powershell
.\abtop.exe --once
.\abtop.exe --json
.\abtop.exe --setup
.\abtop.exe --theme dracula
```

`--setup` installs a Claude Code statusline hook using PowerShell so abtop can read local rate-limit data.

## What This Windows Port Changes

- Uses `sysinfo` for Windows process information.
- Uses `netstat -ano` for listening port discovery.
- Uses native Windows process termination instead of Unix `kill`.
- Uses PowerShell setup/update paths instead of shell scripts.
- Keeps the upstream TUI and collection model as close as possible.

## Notes

- Built for 64-bit Windows.
- No WSL required.
- The binary is not code-signed, so Windows SmartScreen may warn on first run.
- abtop is read-only except for config/cache/setup files it writes in your user profile.

## Upstream Project

This is a Windows-focused fork of:

[graykode/abtop](https://github.com/graykode/abtop)

For the full feature list, design background, themes, and non-Windows usage, see the upstream repository.

## Privacy

abtop reads local process state and local agent session files. It does not require API keys or authentication.

The JSON snapshot includes local paths, command previews, `summary`, `chat_messages`, and recent chat-derived metadata. Treat `--json` output as private local data.

## Disclaimer

This project is provided "as is", without warranty of any kind. Use it at your own risk. The authors and contributors are not liable for any claims, damages, or other liability arising from use of this software.

## License

MIT. See [LICENSE](LICENSE).
