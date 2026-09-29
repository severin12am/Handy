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

### Not in this release

See UPSTREAM.md. Meeting mode, binding dictation to a specific window, and a live microphone meter in settings are not finished. Several upstream issues already have open pull requests and were left to those PRs rather than reimplemented here.
