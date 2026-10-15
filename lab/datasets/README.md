# Jeux de situations

Une ligne JSON = une situation : ce qu'il y avait à l'écran, les actions possibles, ce que l'utilisateur voulait vraiment faire.

```json
{"id": "s01", "app_bundle_id": "com.apple.mail", "window_title": "Re: soutenance",
 "selected_text": "Peux-tu confirmer ta présence jeudi ?", "url": null,
 "available": ["reply.draft", "reply.send", "text.summarize", "..."],
 "expected": ["reply.draft"], "profile": "default", "annotator": "B", "notes": ""}
```

| Champ | Sens |
|---|---|
| `available` | les actions que le catalogue propose dans cette situation |
| `expected` | la ou les actions jugées correctes par l'annotateur (plusieurs si c'est ambigu) |
| `profile` | l'utilisateur dont l'historique compte (sert à mesurer la personnalisation) |
| `annotator` | qui a annoté (A, B, C ou D) |

## Règles

- **Rien de réel n'est commité sans anonymisation** : noms, adresses, contenus de mails. Les captures brutes vont dans `private/`, ignoré par git.
- Chaque situation est annotée par la personne qui l'a vécue ; les cas ambigus sont relus par une deuxième personne.
- `situations.sample.jsonl` sert uniquement à vérifier que le banc tourne : ses scores ne prouvent rien (les règles d'exemple ont été écrites en le regardant).
- Le jeu d'évaluation (au moins 50 situations, v0.3) est figé avant de régler les backends. On l'étend par versions (`situations.v2.jsonl`) ; on ne le modifie jamais en place, sinon les mesures d'avant et d'après ne se comparent plus.
