# Upstream pull requests

This fork is meant to be submitted back to [cjpais/Handy](https://github.com/cjpais/Handy) as small PRs. Each item below is one topic. Open them against `cjpais/Handy` `main`, one branch each, and keep the MIT license header untouched. Do not open a second PR when one is already listed.

Base every branch on current `cjpais/Handy` `main` and cherry-pick only the files named here. `settings.rs`, `actions.rs`, `transcription.rs`, `clipboard.rs`, and `lib.rs` contain more than one topic; split those hunks when you open the PR.

## Ready to open

### 1. Native microphone format

- Issues: cjpais/Handy#2141, #2028
- Files: `src-tauri/src/audio_toolkit/audio/recorder.rs`, `src-tauri/src/audio_toolkit/audio/recorder/tests.rs`
- Also related: open PR cjpais/Handy#2144 ("just use default input config"). Prefer reviewing that PR. This fork keeps the config cache and still uses `default_input_config()` instead of scoring F32 highest. Do not open a competing PR if #2144 lands.
- Suggested title: `fix(audio): capture in the device's native sample format`
- Suggested body: Windows endpoints such as Realtek UAD open an F32 stream and then deliver zeros. Use the default shared-mode config and convert to f32 in the callback. Fixes #2141, helps #2028.

### 2. Unicode direct paste

- Issues: cjpais/Handy#439, #1853, #2126 (direct-input half)
- Files: `src-tauri/src/input.rs`, the `Win32_UI_Input_KeyboardAndMouse` feature in `src-tauri/Cargo.toml`
- Suggested title: `fix(windows): type direct paste with KEYEVENTF_UNICODE`
- Suggested body: `enigo.text` follows the active layout, so Direct paste drops Cyrillic and letters such as ê/œ. Send Unicode input, in short batches so long transcripts are not truncated. Fixes #439, #1853, and the direct-paste case of #2126.

### 3. Clipboard race

- Issue: cjpais/Handy#502
- Files: `src-tauri/src/clipboard.rs` (read-back wait and longer restore), `src-tauri/src/settings.rs` (schema 3 turns `reliable_paste` on once)
- Suggested title: `fix: paste the transcript instead of the previous clipboard`
- Suggested body: The legacy path restores the clipboard on a timer, so a busy target pastes the old contents. Wait until the clipboard echoes the transcript, and turn on the existing receipt-sequenced paste by default. The debug toggle remains an opt-out. Fixes #502.

### 4. Long recordings

- Issue: cjpais/Handy#1332
- Files: `src-tauri/src/audio_toolkit/text.rs` (`chunk_audio_*`, `join_transcript_chunks`), `src-tauri/src/managers/transcription.rs` (`run_chunked`), ring size in `recorder.rs`
- Suggested title: `fix: transcribe long recordings in chunks instead of dropping them`
- Suggested body: Audio longer than 25 seconds is split and transcribed in order. The capture ring is 8 seconds so a slow consumer drops less. A transcription error still leaves the WAV for retry. Fixes #1332.

### 5. Silence warning

- Issue: cjpais/Handy#1899
- Note: open PRs cjpais/Handy#2150 and #1903 already do this. Do not open another if either is still active. This fork records the raw peak and emits `handy:no-microphone-audio` / `handy:no-speech`.
- Files: peak tracking in `recorder.rs`, `managers/audio.rs`, the empty-audio branch in `actions.rs`, toast in `src/App.tsx`, `errors.noMicrophoneAudio*` in locales.

### 6. `<unk>` tokens

- Issue: cjpais/Handy#2145
- Files: `strip_unk_tokens` in `audio_toolkit/text.rs`, call site in `transcription.rs`
- Suggested title: `fix: strip <unk> tokens from transcripts`
- Suggested body: Fixes #2145.

### 7. History limit data loss

- Issue: cjpais/Handy#1262
- Files: `src-tauri/src/commands/history.rs`, `src/components/settings/HistoryLimit.tsx`
- Suggested title: `fix: don't delete recordings while the history limit is being edited`
- Suggested body: Commit the limit on blur, and only prune when the new limit is smaller under count-based retention. Setting auto-delete to Never does not delete. Fixes #1262.

### 8. API keys from the environment

- Issue: cjpais/Handy#483
- Files: `src-tauri/src/secrets.rs`, call sites in `actions.rs` and `shortcut/mod.rs`, `Win32_Security_Credentials` in `Cargo.toml`
- Suggested title: `feat: read post-processing API keys from the environment`
- Suggested body: Empty settings keys fall back to `HANDY_<PROVIDER>_API_KEY` and `HANDY_POST_PROCESS_API_KEY`. On Windows a saved key is also copied to Credential Manager. Fixes #483.

### 9. Asset protocol scope

- Issue: cjpais/Handy#1384
- Note: open PR cjpais/Handy#2021. Coordinate with it.
- Files: `src-tauri/tauri.conf.json`, `allow_directory` in `lib.rs` setup.

### 10. Process stays open after quit

- Issue: cjpais/Handy#1005
- Files: quit arm and `bind_kill_on_close_job` in `lib.rs`, `Win32_System_JobObjects` in `Cargo.toml`
- Suggested title: `fix(windows): exit the process on Quit`
- Suggested body: `app.exit` left hook and WebView2 processes running. Unload the model, exit the process, and put it in a kill-on-close job. Fixes #1005.

### 11. Overlay compositor while hidden

- Issues: cjpais/Handy#2162, #1371
- Files: `set_overlay_webview_visible` in `overlay.rs`
- Suggested title: `fix(windows): stop the hidden overlay webview from compositing`
- Suggested body: Hiding the HWND is not enough; WebView2 keeps a timer. Set the controller invisible when the overlay hides. Fixes #2162 and #1371.

### 12. Slow GPU reload and GPU failure fallback

- Issues: cjpais/Handy#1841, #1755, #2047, #2129
- Files: `STREAM_FINALIZE_REPLY_TIMEOUT` and `load_whisper_model_with_cpu_fallback` in `transcription.rs`
- Suggested title: `fix: don't drop a transcript when the GPU backend is slow or broken`
- Suggested body: Finalize waits 3 minutes so a ~70s Vulkan reload can finish. A GPU load that fails or panics retries on CPU and saves that choice. Fixes #1841, #1755, #2047, #2129. A driver BSOD cannot be caught in process; CPU is remembered only after a failure we can observe.

## Features (discuss before opening; Handy is in a feature freeze)

### Smart spacing, voice commands, Russian fillers, per-app auto-submit

- Files: `audio_toolkit/text.rs`, settings fields `smart_spacing`, `voice_commands_enabled`, `auto_submit_apps`, commands in `shortcut/mod.rs`, `src/components/settings/SmartSpacing.tsx`, `VoiceCommands.tsx`, `AutoSubmit.tsx`, locale strings.
- Suggested discussion: these extend existing filler removal, trailing space, and auto-submit. They do not add a second post-processing or dictionary system.

## Skipped on purpose

| Topic | Why |
| --- | --- |
| #642 mute delay | Open PR cjpais/Handy#2168 |
| #1899 if #2150 or #1903 merges | Open PRs already |
| #2141 if #2144 merges | Open PR already; this fork's version keeps the config cache |
| #1384 if #2021 merges | Open PR already |
| #2089 first hold-or-toggle edge | Open PR cjpais/Handy#2097 changes the same timing path; not reimplemented here |
| #508 overlay state | No reliable repro beyond the hide/show generation fix already on main. WebView visibility in #11 may help. |
| #1314 hotkey swallow | Needs a captured trace of which hook ate the key. Not changed. |
| #1884 performance since 0.9.4 | No local profile yet; the machine could not finish a Vulkan build (SDK installer was cancelled at the UAC prompt). |
| Meeting mode | Not implemented. Needs a WASAPI loopback path and a transcript window; Handy has no loopback capture to extend. |
| Target-window binding | Not implemented. Needs a foreground-override that is easy to get wrong on Windows. |
| Microphone level meter in settings | Not implemented. The overlay visualizer and the silence toast cover the diagnostic the issues asked for; a settings meter is still worth adding. |
| #2070 focus loss after 3s | Not reproduced in code review; no change. |

## Verification already run

On this Windows machine, `cargo test --lib` for the text helpers, settings salvage/migration, and recorder unit tests: 26 passed after the look-ahead regex fix (the locked `regex` crate has no look-around). A full `tauri build` with Vulkan did not run: the LunarG SDK installer asked for administrator and the prompt was cancelled. GPU behavior is therefore not verified on the RTX 4070 yet.
