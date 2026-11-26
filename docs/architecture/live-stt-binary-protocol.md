# Experimental Live STT binary protocol

R687 replaces the R686 JSON/text WebSocket messages with a single compact binary protocol. This is a development-phase break, not a backward-compatible negotiation. The endpoint remains `ws://127.0.0.1:8799/v1/stt/live`, restricted by the existing loopback and Origin checks.

Each WebSocket **binary message** is one complete application frame. The first byte is its kind. Integer fields are little-endian. The client sends 16 kHz, mono, signed PCM16 without a WAV header. No JSON, Base64 or serializer crate is involved. UTF-8 is used only for transcript content.

| Direction | Kind | Payload |
|---|---|---|
| Client to server | `0x01` | `u32` sample rate, must be 16000 |
| Client to server | `0x02` | No fields; finish utterance |
| Client to server | `0x03` | 2 to 6400 bytes of interleaved little-endian PCM16 samples, mono |
| Server to client | `0x81` | `u32` rate 16000, `u16` maximum seconds 30, `u8` mode 1 for Qwen snapshot recognition |
| Server to client | `0x82` | No fields; ready for PCM |
| Server to client | `0x83` | `u32` cumulative sample count, followed by UTF-8 partial transcript |
| Server to client | `0x84` | `u32` cumulative sample count, followed by UTF-8 final transcript |
| Server to client | `0x85` | One byte error code |

Error codes: 1 no speech; 2 inference unavailable; 3 recognition task failed; 4 session timeout; 5 invalid session transition; 6 unexpected message type; 7 session audio limit; 8 invalid audio frame; 9 other rejected session. The server then sends a WebSocket close frame.

Limits: two concurrent sessions, 30 seconds audio per session, 45-second session timeout, and one in-flight snapshot transcription per session. Recognition is still periodic Qwen3-ASR on accumulated snapshots, not a checkpoint-compatible true Zipformer streaming encoder.

The Yew Live tab uses an `AudioWorklet` to transfer PCM blocks off the audio rendering thread, incrementally resamples to 16 kHz and sends bounded 100 ms frames. It flushes the AudioWorklet pending block before sending Finish, then stops capture. It also stops capture when leaving the tab, on explicit cancellation, or when replaced. If the socket's `bufferedAmount` exceeds 64 KiB, the browser reports an error rather than silently dropping arbitrary speech. The live audio is not saved to disk by the new transport.

This protocol has no compatibility with the earlier R686 JSON messages. The client and server must both come from R687. The `AudioWorklet` uses a generated local blob module and needs a browser/WebView security-policy check; the hosted `/zipeek/` deployment cannot connect to the localhost speech server from an untrusted origin.

## Acceptance

Run `cargo check --locked -p zipeek-tts-stt`, `cargo test --locked -p zipeek-tts-stt --test live_stt_websocket`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, `cargo test --locked -p zipeek-browser` and `cargo test --locked -p zipeek-web` on the Windows development machine. Start the localhost speech service with a Qwen ASR checkpoint and `ZIPEEK_STT_DEVICE=cuda`. In the local app, open `#/record`, select Live, start the microphone, verify real PCM waveform and partial transcripts, finish and verify the final transcript. Repeat cancellation, no speech, wrong origin, reconnect, closed socket, and mode switching. Verify browser and Tauri WebView CSP support for the worklet module.
