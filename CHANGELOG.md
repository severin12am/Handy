# Changelog

Community fork of [Handy](https://github.com/cjpais/Handy) by cjpais. Each item names the upstream issue it addresses. The MIT license and original copyright are unchanged.

## Unreleased

### Bug fixes

- **cjpais/Handy#502** — Clipboard paste waits until the transcript is actually on the clipboard before sending the paste chord, and receipt-sequenced paste (restore only after the target reads the clipboard) is on by default. The previous clipboard is still restored afterwards.
- **cjpais/Handy#439 / #1853** — Direct paste on Windows uses `SendInput` with `KEYEVENTF_UNICODE`, so Russian and accented characters (ê, œ) are not dropped by the active keyboard layout.
- **cjpais/Handy#2126** — Long direct pastes are sent in short batches so the target application's input queue does not drop the tail. Clipboard paste keeps the full transcript in one payload and holds the restore longer for long strings.
- **cjpais/Handy#2141 / #2028** — Capture uses the microphone's native shared-mode format instead of forcing float32. Samples are converted to float internally. This is the Realtek/Windows case where the stream opens and then delivers digital silence.
- **cjpais/Handy#1899** — Digital silence (and a recording with no speech) shows a clear message instead of pasting nothing.
- **cjpais/Handy#1332** — Recordings longer than 25 seconds are transcribed in chunks split near quiet audio, instead of being dropped. The capture ring is 8 seconds so a slow consumer is less likely to discard audio. A failed transcription still keeps the WAV for retry.
- **cjpais/Handy#2145** — Literal `<unk>` tokens are removed from the transcript.
- **cjpais/Handy#1262** — History limit commits when the field is left (not on every keystroke) and only deletes recordings when the limit actually shrinks. "Never" auto-delete does not delete.
- **cjpais/Handy#2139** — A settings file with an invalid field still loads; only that field falls back to its default (already true on current Handy; kept and covered by tests).
- **cjpais/Handy#483** — Post-processing API keys are read from `HANDY_<PROVIDER>_API_KEY` or `HANDY_POST_PROCESS_API_KEY` when the settings field is empty. On Windows, saved keys are also copied into Credential Manager.
- **cjpais/Handy#1384** — The asset protocol can read the app's own data folders, not the whole filesystem.
- **cjpais/Handy#1005** — Quit unloads the model and exits the process. On Windows the process is in a job that kills WebView2 children on exit.
- **cjpais/Handy#2162 / #1371** — Hiding the recording overlay also hides the WebView2 controller so a hidden compositor does not keep a high-resolution timer while the machine is locked or the lid is closed.
- **cjpais/Handy#1841** — Live transcription waits up to three minutes to finish, so a slow Vulkan reload after idle unload does not time out and drop the text.
- **cjpais/Handy#1755 / #2047 / #2129** — If the GPU backend fails or panics while loading a Whisper model, Handy retries on CPU and remembers that choice.

### Features

- Smart spacing: a space after `. , ! ? : ;` when a letter follows (decimals stay intact), and a single leading space between back-to-back dictations. On by default.
- Voice commands (off by default): "new line" / "новая строка" and "new paragraph" / "новый абзац".
- Russian filler removal when the transcript language is Russian (`ну`, `ээ`, `эээ`, `эм`, `типа`), using the existing filler-word toggle.
- Auto-submit can be limited to a list of window-title fragments (Telegram, Discord, Cursor, …). An empty list keeps the previous "every app" behavior.
- Custom dictionary words and post-processing prompt presets were already in Handy; this fork does not add a second copy of them.

### More Windows fixes

- **cjpais/Handy#2089** — Hold-or-toggle measures the first shortcut from the key event time, not from when the coordinator thread gets around to it. A short tap during microphone startup no longer stops the recording immediately. Based on PR #2097 by @xronocode. Unit test `auto_mode_tap_survives_first_activation_start_latency`.
- **cjpais/Handy#642** — Mute-while-recording mutes system audio when recording is requested, before the microphone opens, instead of after the start chime. Based on the immediate-mute idea in PR #2168 by @Pifan07.
- **cjpais/Handy#1314** — On Windows, handy-keys listens without swallowing the shortcut, so the keys still reach the focused app. Likely fixed, not reproduced against every desktop shortcut.
- **cjpais/Handy#508** — Overlay show/hide events carry a generation. A late "recording" or "hide" from an earlier session is ignored. Likely fixed, not reproduced (the report was an elevated Visual Studio).
- **cjpais/Handy#2070** — If the capture stream dies mid-take, Handy says so instead of closing with no transcript. Likely fixed, not reproduced.
- **cjpais/Handy#1884** — When Auto binds an integrated GPU (Intel/UHD/Iris/Arc, or a Radeon that is not an RX card), Handy logs that CPU may be faster. Discrete NVIDIA stays on the GPU. Likely relevant, not reproduced on this RTX 4070.
- **cjpais/Handy#1899** — Already implemented in this fork (peak check and toast). Upstream PRs #2150 by @dimmgigoveu-gif and #1903 by @Charlie284 cover the same bug; this fork does not open another PR.
- **cjpais/Handy#2141** — Already implemented by using the device default format and keeping the config cache. Based on the approach in PR #2144 by @cjpais. This fork does not open a duplicate PR.
- **cjpais/Handy#1384** — Asset protocol stays limited to app data. Based on PR #2021 by @BradGroux, which restricts it to the recordings directory; this fork allows the app data folder so portable mode and recordings both work, and does not open a duplicate PR.

### Target window

- Bind dictation to an open window (process name, then title). Pick from a list or grab the window focused 3 seconds later.
- Paste focuses that window (SetForegroundWindow with AttachThreadInput and AllowSetForegroundWindow), then restores the previous focus.
- Minimized windows are restored. If the window is gone, the transcript is copied to the clipboard and a toast says it was not found.
- Optional Enter after paste for that binding. Tray item and Ctrl+Alt+B toggle it. The overlay shows the process name.
- Windows only. The control is hidden on macOS and Linux.

### Microphone meter

- Sound settings show a level bar and **Test microphone**. It listens for about a second in the device's native format and says whether that was real signal or digital silence.

### Not in this release

See UPSTREAM.md. Meeting mode, binding dictation to a specific window, and a live microphone meter in settings are not finished. Several upstream issues already have open pull requests and were left to those PRs rather than reimplemented here.
