# AI Hanzi recognition

## Purpose and route

Provide a dedicated **Recognize Hanzi** route for identifying unknown single Chinese characters or multi-character words from handwriting or images. Recognition supplies possible matches, not a guaranteed transcription. It is distinct from Write Hanzi stroke-order grading and must not award pedagogical evidence merely because a candidate was recognized.

## Unified canvas

Use **one canvas**, without separate Draw and Paste tabs. Support pointer, touch and stylus drawing; clipboard image paste; and image file selection from a compact toolbar action. Show clear empty, drawing, image-loaded, busy, error and result states. Support Undo and Clear for drawing. If annotation over a pasted image is enabled, keep original image pixels separate from editable stroke overlays so Undo does not destroy the input. Replacing an existing drawing or image requires an explicit user action.

Clipboard handling is capability-aware. A permission-enabling action may request image clipboard access when the browser and security context support it. A component lifecycle hook may register focus and paste listeners and clean them up, but **must not auto-prompt for permission on mount** or poll the clipboard continuously. Chromium's permission behavior does not apply universally to Firefox and Safari. Ordinary paste and file upload must remain usable without persistent clipboard permission. A detected image is offered as a contextual action, not pasted automatically over existing work.

The app must not transmit clipboard content just because a clipboard image is present. Upload to a configured vision-capable AI service requires the learner to explicitly press **Recognize**. Check service capabilities and availability before enabling recognition. Handle cancellation, stale responses, request size limits, unsupported image types, empty drawings, provider failure and malformed model output.

## Recognition and result validation

Normalize image dimensions and excess whitespace non-destructively. Keep enough visual context for multi-character text. Request a small ranked set of candidate Hanzi or vocabulary strings from the vision service. Verify dictionary hits through the existing dictionary lookup rather than trusting model-generated pinyin or meanings. Mark unsupported candidates as unverified. If characters are visually ambiguous, show alternatives with readable labels rather than claiming certainty from an uncalibrated score.

Provide Listen when the existing Mandarin audio service is available, Open in Dictionary, Add to the active Word Set, and Practise writing where valid. A failed dictionary match must not be silently promoted to a verified entry.

## Privacy and retention

Do **not** persist recognition history, source images or recognition results. Keep input and candidates only in the current interaction state; release them on Clear, navigation or unmount. Never upload automatically. No image data belongs in analytics, saved profiles or crash logs. Respect configured service privacy disclosures and browser permissions. See [AI service architecture](../../architecture/modules/ai-client.md) and [acceptance](../../testing-quality/acceptance/approved-behavior-acceptance.md).

## R532 implementation status

The initial route and single workspace are authored in `zipeek-web/src/pages/learning/recognize_hanzi.rs` and `recognition_workspace.js`. Drawing, Undo, Clear, focus-scoped image paste, PNG/JPEG/WebP upload, explicit submission, temporary results, a small strictly validated candidate list, local dictionary matching and existing Word Set/audio/writing actions are wired. The AI call sends an image URL data payload to the configured chat-completions endpoint only on button activation. Stale responses cannot update the page after another input event, Clear or unmount; the native browser request may continue because the WASM HTTP client has no explicit abort interface.

Unverified requirements: actual vision-capable model discovery, service-capability gating beyond requiring a configured model, cross-browser clipboard support, detailed candidate quality, cropped/whitespace optimized inputs, live source-image privacy audit and Chrome accessibility. No recognition history or source images are deliberately persisted. This is an unverified WIP implementation, not finished acceptance.

## R536 interface review

Recognition input now occupies a compact, centred canvas card with a single main
Recognize action. The Word Set picker is removed from the empty recognition workspace;
existing current-set context is used only after dictionary-verified results exist.
The provider response parser accepts strictly fenced JSON in addition to raw JSON,
but continues to reject invalid or non-Hanzi candidates. Browser recognition quality
and visual UX still require actual acceptance screenshots and tests.

## R537 theme and failure-diagnostic corrections

The drawing surface uses the active application surface and ink colors in light, dark and System modes, including when the appearance changes while a workspace is mounted. The canvas must remain compact enough for a standard desktop viewport while allowing the page to scroll normally when candidate results extend beyond the viewport. Error feedback must distinguish malformed JSON from invalid Hanzi, empty candidates and upstream request failures. Log only bounded structural response information such as response length, parsing location and classification; never log the submitted image, raw model text, credentials or character candidates.

## R539 interface redesign

The current source presents a responsive two-column writing-and-action workspace on wide viewports, a stacked layout on narrow screens, and candidate cards below. The canvas keeps the active theme surface and ink, and empty/input/error states are contextual. Undo is available only for editable strokes, Clear only for content, and recognition only after content is present and a model is configured. No history is saved. This update has not been visually accepted in Chrome or on mobile.

## R550 responsive results layout

On desktop the writing workspace and candidate results share a two-column layout, with the canvas occupying roughly three-fifths of the available width. The writing workspace remains visible during candidate inspection. Results have their own bounded vertical scroll region on large viewports; do not put scroll containers inside individual cards. On narrow screens the canvas comes first, followed by the candidate list, and successful recognition scrolls the results into view. The initial empty results panel stays quiet without repetitive instructions or fabricated candidate data. A visible busy indicator and accessible results announcement appear only when relevant. The page preserves normal mobile scrolling, theme support, and existing dictionary/Word Set actions. The implementation has not yet passed browser visual acceptance.

## Recognition request and result presentation

Use a single concise JSON-only recognition contract with a bounded output budget. A response with no recognized characters uses an empty `candidates` array, distinct from malformed JSON. The dictionary remains authoritative for pinyin and meaning. Preserve reasoning/output-length errors as actionable failures without logging submitted images or model content.

On desktop, align drawing and suggestions panels in one grid row and keep candidate suggestions concise. Each row includes the candidate, local dictionary pinyin/meaning when available, and accessible actions for listening, dictionary, writing and Word Sets. Keep an ordinary page flow on mobile. Do not repeat the same controls as large text-only action rows.
