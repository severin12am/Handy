# Architecture

Handy is a Tauri 2 desktop app: a React settings UI and a Rust core that records the microphone, runs a local speech model, and inserts the text into whatever app is focused. This note describes the pipeline as it exists in this community fork of [cjpais/Handy](https://github.com/cjpais/Handy). Behavior that this fork changes is called out inline.

## Startup

`src-tauri/src/lib.rs` `run()` builds one Tauri process and refuses a second copy (`tauri-plugin-single-instance`). A second launch forwards CLI flags (`--toggle-transcription`, `--cancel`) to the running instance and exits.

Setup then:

1. Loads `settings_store.json` through `tauri-plugin-store` (`settings.rs`). A store that fails to deserialize is salvaged field-by-field; one bad value does not reset the rest. Schema migrations run once (device ids, overlay style, receipt-sequenced paste).
2. Creates the main window (hidden when "start hidden" or the tray is the only entry point) and a separate always-on-top overlay webview (`overlay.rs`, `src/overlay/`).
3. Builds the tray menu and registers global shortcuts (`shortcut/`).
4. Starts the audio, model, transcription, and history managers.

Closing the main window hides it. Quit is a tray action. On Windows the process is placed in a job object that kills WebView2 children on exit, and Quit calls `process::exit` after unloading the model so background threads cannot keep the process in Task Manager.

Portable mode (`portable.rs`): a `portable` marker file next to the executable stores settings, models, and recordings in `Data/` beside the exe instead of `%APPDATA%`.

## Audio capture

`managers/audio.rs` owns the microphone lifecycle (on-demand or always-on). `audio_toolkit/audio/recorder.rs` opens a cpal input stream on a worker thread.

The stream uses the device's **default shared-mode format** (sample rate and sample type). Forcing float32 when the endpoint's mix format is integer PCM opens successfully on some Windows devices and then delivers zeros (Realtek UAD). The callback converts every sample to `f32` and downmixes channels. A resampler turns that into 16 kHz frames for the model.

Frames pass through optional voice-activity detection (Silero by default, Earshot behind the experimental toggle) in `audio_toolkit/vad/`. Speech frames accumulate for the recording; a peak of the raw samples is kept so digital silence can be reported even when VAD drops every frame.

A ring buffer (8 seconds) sits between the real-time callback and the consumer. If the consumer falls behind, samples are counted as overruns and logged rather than blocking the audio callback.

`Mute while recording` (`managers/audio.rs` `set_mute`) ducks the default render endpoint for the duration of the take.

## Hotkeys

Two backends, selected in debug settings:

- `handy-keys` (`shortcut/handy_keys.rs`) — a low-level hook. It is the default on Windows.
- Tauri's global shortcut plugin (`shortcut/tauri_impl.rs`).

`transcription_coordinator.rs` turns key down/up into a recording. `ShortcutActivation` is hold, toggle, push-to-talk, or "hold or toggle" (a short press toggles, a longer press is hold). The first edge after startup is classified from the key event's own timestamp.

Cancel is registered only while a recording or transcription is in flight. Shortcuts the user is currently editing are suspended so the hook does not swallow the keys being bound.

## Transcription pipeline

`actions.rs` `TranscribeAction` is the shortcut handler.

1. `start` kicks a model load if needed, opens the mic, shows the overlay, and (for streaming-capable models) feeds audio to a live stream as it arrives.
2. `stop` drains the recorder, writes a WAV into the history folder, and transcribes.
3. `managers/transcription.rs` runs the audio through the loaded engine.

Whisper-family GGUF models go through `transcribe-cpp` (Vulkan on Windows x64, Metal on macOS, CPU fallback). ONNX models (Parakeet, Moonshine, SenseVoice, GigaAM, Canary, Cohere) go through `transcribe-rs`. The accelerator setting is Auto, GPU, or CPU. If a GPU load fails or panics with a device/Vulkan error, the same load is retried on CPU and the CPU choice is saved so the next launch does not hit the broken device again.

Audio longer than 25 seconds is split on quiet boundaries and transcribed in pieces. A failure returns an error; the WAV is still saved and the history row can be retried. The live-stream finalize wait is long enough for a cold Vulkan reload (shader compile after an idle unload) instead of giving up at 30 seconds and dropping the text.

After the model returns text, `post_process_transcription_text`:

- fuzzy-corrects custom dictionary words (`audio_toolkit/text.rs`) when they were not already passed as a Whisper initial prompt
- removes filler words (including Russian ээ / ну / типа when the output language is Russian), unless the user turned that off
- strips `<unk>` tokens
- optionally applies offline voice commands ("new line", "новая строка", "new paragraph", "новый абзац")
- optionally inserts a space after `. , ! ? : ;` when a letter follows, without splitting decimals

If the captured peak is digital silence, nothing is pasted. The overlay closes and the UI shows "No audio from the microphone".

## Post-processing

Optional and off by default. `llm_client.rs` sends the transcript to an OpenAI-compatible HTTP API (or the built-in provider list). Prompts are user-editable presets in settings; one is selected and can be triggered by the separate "transcribe with post-processing" shortcut. API keys are read from the settings store, then Windows Credential Manager, then `HANDY_<PROVIDER>_API_KEY` / `HANDY_POST_PROCESS_API_KEY`. The app works fully offline when this is disabled.

## Text insertion

`clipboard.rs` `paste()` runs on the UI thread after transcription.

Paste methods:

| Method | What it does |
| --- | --- |
| Clipboard (`Ctrl+V`, `Ctrl+Shift+V`, `Shift+Insert`) | Writes the transcript, then sends the chord |
| Direct | Types the text |
| None | Leaves the text in history only |
| External script | Hands the text to a user-supplied program |

On macOS and Windows the clipboard path prefers **receipt-sequenced paste** (`paste_tx/`). The transcript is published as a delayed-render clipboard promise. The previous clipboard (text or image) is snapshotted and restored only after the target application actually reads the promise, or after a bounded timeout. A fixed sleep loses the race under load and pastes the old clipboard. The legacy path remains as a fallback and now waits until a clipboard read-back matches the transcript before sending the chord, then holds the restore longer for long strings.

Direct input on Windows is `SendInput` with `KEYEVENTF_UNICODE`, so the active keyboard layout does not matter (Russian, French œ/ê). Long strings are injected in short batches so the target's input queue does not drop the tail.

Smart spacing may prepend one space when the previous dictation (within 20 seconds) ended on a non-space character, and never produces a double space. Trailing-space remains its own setting.

Auto-submit optionally presses Enter (or Ctrl/Super/Cmd+Enter) after paste. A per-app list limits that to foreground windows whose title contains one of the names (Telegram, Discord, Cursor, …). An empty list keeps the old "every app" behavior.

## Overlay

`overlay.rs` creates a transparent, non-focusable, always-on-top window. React (`src/overlay/RecordingOverlay.tsx`) shows a recording pill or a live transcript. Position is top or bottom; style is minimal, live, or hidden.

Mic levels are emitted only while the overlay is enabled, and throttled. On Windows, hiding the overlay also sets the WebView2 controller invisible so a hidden compositor does not keep a 1 ms timer running while the machine is locked or the lid is closed.

## Settings store

`settings.rs` `AppSettings` is one JSON object under the key `settings` in `settings_store.json`. Every field has a serde default. `get_settings` migrates old stores and, if the whole object fails to parse, rebuilds it from the fields that are individually valid. The React side is a Zustand store (`src/stores/settingsStore.ts`) that calls one Tauri command per field. Commands are typed by specta into `src/bindings.ts`.

## History

`managers/history.rs` stores each take in SQLite (`transcription_history`) plus a WAV under the recordings directory. History limit and auto-delete are separate. Count-based deletion runs only when the retention mode is "preserve limit" **and** the saved limit got smaller. Switching retention to "Never" does not delete. The limit field commits on blur, not on each keystroke, so typing a new number cannot wipe recordings mid-edit. A failed transcription still saves the WAV and an empty row so the retry button in History can run it again.

## Updater

`tauri-plugin-updater` checks the endpoint in `tauri.conf.json` when update checks are enabled. `HANDY_DISABLE_UPDATER=1` forces checks off without rewriting the setting (used by package managers). The UI lives in `src/components/update-checker/`.

## Asset protocol

The webview loads recording WAVs through Tauri's asset protocol (`convertFileSrc`). The scope is the app data directory, local app data, and bundled resources — not the whole filesystem. Portable mode's `Data/` directory is allowed at startup.

## Frontend

`src/App.tsx` is the settings shell: onboarding, sidebar sections, toasts for paste / transcription / microphone failures. Strings go through i18next (`src/i18n/locales/`). The overlay is a second Vite entry (`src/overlay/index.html`) so it can be shown without focusing the settings window.
