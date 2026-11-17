# Sentence Relay

## Product contract

Sentence Relay is a Chinese word-order game. The player assembles a Chinese sentence matching an English prompt, receives contextual correction feedback, and progresses through the configured number of sentences. These are approved target requirements, not claims about the currently shipped implementation.

## Game stages

1. **Setup:** Select the vocabulary source and intended sentence count, review relevant game options and the availability of the configured AI sentence generator. Starting a game is an explicit action.
2. **Preparing:** After Start game, show preparation progress and a cancel action. Do not simultaneously present a disabled Start game button and a generation message without explaining the state. No AI generation or associated service usage should start implicitly when the user is still choosing settings.
3. **Play:** Present the English target, Chinese answer area, word bank, controls, completed-sentence progress and mistakes. Hide setup-only controls. A compact, labelled settings icon opens the setup transition; warn before abandoning meaningful progress.
4. **Results:** Present sentence-level outcomes and mistakes, with actions to play again or change setup. Keep gameplay-specific evidence separate from general Practice mastery claims.

The sentence-count setting belongs in setup and determines the generation request, progress denominator and finish condition. The exact preset values and default are pending confirmation; do not treat suggested values as a final product decision. If the service returns fewer valid sentences than requested, explain the shortage rather than silently displaying a different count.

## Word bank and interaction

- The word bank contains Chinese word or phrase tokens, not free-standing punctuation marks. Render Chinese punctuation in the appropriate position as part of sentence presentation and validation, including commas or question marks when relevant. Do not make a full stop a shuffled token.
- Support clicking or tapping to place a token; dragging to place it at a chosen answer position; and reordering tokens within the answer. Provide equivalent keyboard-accessible operations and explicit focus states.
- A placed token disappears from the available word bank rather than remaining as a faded duplicate. Undo or removing a token returns it to the bank.
- Provide a configurable pronunciation-on-selection option. When enabled, selecting a word should play its pronunciation; dragging should play once on drop, not continuously while moving. Reuse the shared pronunciation service and prevent overlapping audio. The option's initial default remains to be confirmed.
- The answer area clearly indicates how to build an answer when empty. Preserve answers during correction and allow editing without clearing the whole sentence.
- Count mistakes on incorrect submitted attempts, not on individual token actions. Progress measures completed sentences rather than placed words.

## Contextual correction feedback

A generic message such as "Not quite. Edit your sentence and try again." is insufficient when the system can identify a specific word-order error. The feedback must explain what kind of correction is needed and indicate the relevant tokens without reflexively revealing the entire answer.

For the prompt **He is eating now**, a submitted arrangement of **他 在 吃 饭 正** with the expected arrangement **他 正 在 吃 饭** illustrates a moved-token error. Highlight **正** and, when the validator has reliable evidence, indicate that it belongs before **在**. Keep the learner's answer visible and editable.

- If the right tokens are present in the wrong order, identify a misplaced token or phrase using sequence-aware comparison instead of flagging every later token by positional index.
- If a word is missing, indicate the missing position or needed grammatical component without immediately filling it in.
- If an extra token is present, identify that token.
- If multiple problems exist, prioritize a small, actionable subset; progressively increase hint specificity on subsequent incorrect submissions.
- If the submitted answer is correct, provide affirmative feedback and advance through an explicit or accessible transition, with optional spoken sentence playback according to audio settings.
- Valid alternate Chinese word orders must not be rejected solely because they differ from a single generated reference. Use suitable grammar-aware validation when available. When the validator cannot determine a specific grammatical fault reliably, give truthful, less specific feedback rather than fabricating an explanation.
- Retain highlighted errors until relevant tokens are edited. Make corrections perceivable with outlines, labels and accessible live announcements, not color alone. Feedback must not interrupt or overlap pronunciation inappropriately.

## Progress and chrome

Use a compact purple progress indicator with a visibly distinct track and accessible textual label. A thick uniform gray bar must not look like a disabled control. A settings icon replaces the prominent text label "Change setup" and carries an accessible name and tooltip. Keep the game's established dark violet cards, readable labels and compact spacing.

## Verification boundary

The screenshot review establishes shortcomings and target UX; it does not establish actual implementation mechanisms. Validate both pointer and keyboard input, mobile layouts, AI success/failure/loading states, alternative word orders, repeated mistakes, sound playback, progress, and results in a browser before marking acceptance complete.
