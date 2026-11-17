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
