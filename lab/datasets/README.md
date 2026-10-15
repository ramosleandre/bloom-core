# Situation sets

One JSON line = one situation: what was on screen, the possible actions, what the user actually wanted to do.

```json
{"id": "s01", "app_bundle_id": "com.apple.mail", "window_title": "Re: defense",
 "selected_text": "Can you confirm you'll attend on Thursday?", "url": null,
 "available": ["reply.draft", "reply.send", "text.summarize", "..."],
 "expected": ["reply.draft"], "profile": "default", "annotator": "B", "notes": ""}
```

| Field | Meaning |
|---|---|
| `available` | the actions the catalog offers in this situation |
| `expected` | the action(s) the annotator judged correct (several if it is ambiguous) |
| `profile` | the user whose history counts (used to measure personalization) |
| `annotator` | who annotated (A, B, C or D) |

## Rules

- **Nothing real is committed without anonymization**: names, addresses, email contents. Raw captures go in `private/`, ignored by git.
- Each situation is annotated by the person who lived it; ambiguous cases are reviewed by a second person.
- `situations.sample.jsonl` is only used to check that the bench runs: its scores prove nothing (the sample rules were written while looking at it).
- The evaluation set (at least 50 situations, v0.3) is frozen before the backends are tuned. It is extended through new versions (`situations.v2.jsonl`); it is never modified in place, otherwise measurements from before and after can no longer be compared.
