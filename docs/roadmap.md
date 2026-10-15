# Roadmap

> Source de vérité de l'avancement de Bloom. Si un kanban existe, il recopie ce fichier, et ce fichier gagne en cas de désaccord.
> Une case cochée = la définition de « fini » d'`AGENTS.md` est remplie.
> Tags : qui `[A]` `[B]` `[C]` `[D]` `[Tous]` ; preuve pour la track `{O1}` `{O2}` `{PERF}` `{EXP}`. Les échéances de la track sont en gras ; les preuves sont rangées dans [`evidence/`](evidence/README.md).

## Où on en est

- Version en cours : **v0.0, socle**
- Prochaine échéance de la track : **fin octobre**, la note sur le problème dur et le dispositif de veille {O1}
- Mis à jour le : 2026-10-15

## Le principe

Une tranche verticale qui marche, puis on mesure, puis on sécurise, puis on intègre la meilleure solution mesurée. Une version ne commence pas tant que la démo de la précédente ne tourne pas.

---

## v0.0 · Socle (15 → 31 octobre)

But : tout le monde peut coder, la CI garde la porte, les deux livrables d'octobre partent.

- [x] [A] Squelette du dépôt : corps, mocks des 4 blocs, tests, banc de décision, CI, base de connaissance
- [ ] [A] Dépôt GitHub créé, `main` protégée, CI active {O2}
- [ ] [Tous] `make setup`, `make check` et `make run` verts sur les 4 machines
- [ ] [Tous] Relire les ADR 0001 à 0006, un commentaire chacun au minimum, puis les passer en « accepté » {O2}
- [ ] [Tous] Le cas d'usage de référence écrit en une phrase, et les 8 actions listées avec leur plancher
- [ ] [A] Mesurer l'approche naïve (un LLM génératif classe les 8 actions) : 30 appels, p50 et p95 {O1}
- [ ] [A] Finaliser [`hard-problem.md`](hard-problem.md) avec les seuils chiffrés, l'envoyer au mentor {O1}
- [ ] [Tous] Finaliser [`evidence/research-setup.md`](evidence/research-setup.md) : sources, outils en place, rythme, répartition {O1}
- [ ] [Tous] Une première entrée de veille chacun dans `evidence/research-log.md` {O1}
- [ ] [C] Tester la permission Accessibilité depuis un sidecar Python lancé par Tauri (risque n°1 de l'architecture)

**Démo de fin** : `make check` et `make run` passent chez tout le monde ; le mentor a reçu la note sur le problème dur.

## v0.1 · Le corps tourne sur des faux blocs (1 → 15 novembre)

But : chacun peut développer son bloc seul, contre les mocks des autres.

- [ ] [A] `contracts.py` v1 figé, relu par les 4 (label `contrat`)
- [ ] [B] [C] [D] Chaque bloc a son mock fidèle au contrat et ses tests de conformité
- [ ] [A] Un seul run actif à la fois : un nouveau raccourci annule le précédent
- [ ] [A] Événements `RunFinished` et `Feedback`, ajoutés quand leur premier consommateur existe
- [ ] [Tous] Rituel de partage interne en place, première présentation notée {O1}
- [ ] [Tous] Première review qui fait changer une PR, notée dans `evidence/reviews.md` {O2}

**Démo de fin** : le test d'intégration couvre les quatre issues d'un run, et chaque bloc tourne seul contre les mocks.

## v0.2 · Tranche verticale réelle (16 novembre → 20 décembre)

But : sur le Mac de référence, raccourci, menu, action exécutée, pour de vrai.

- [ ] [C] `observe()` réel sur macOS : application, fenêtre, sélection
- [ ] [C] Les 8 actions avec leur plancher ; `execute()` réel pour au moins 3, dont une `EXTERNAL`
- [ ] [B] Contexte réel : fragments de l'appareil et court terme, étiquetés, coupés au budget, sans réseau
- [ ] [A] Un backend System 1 (Jev ou Laya) derrière `DecisionPort`, à côté des règles
- [ ] [A] `generate()` (LLM) branché pour une action qui écrit
- [ ] [D] Shell Tauri : raccourci global, pie menu (positions fixes + emplacements contextuels), fenêtre de validation
- [ ] [D] Pont IPC (WebSocket local, jeton, Origin) et types TypeScript générés depuis les contrats
- [ ] [A] Schéma d'architecture et schéma de données à jour dans `architecture.md` · **fin novembre** {O2}
- [ ] [A] [Tous] Note d'état de l'art : au moins 3 approches comparées, sources datées · **fin décembre** {O1}
- [ ] [D] README, doc de l'API (messages IPC) et doc d'exploitation, première version · **fin décembre** {O2}
- [ ] [A] Rapport qualité de référence : `make quality` · **décembre** {O2}
- [ ] [A] [D] Note de choix PERF : axe, indicateurs, cible · **fin décembre** {PERF}
- [ ] [A] Note sur les 2 ou 3 sujets à faire challenger par des experts · **fin décembre** {EXP}

**Démo de fin** : la tranche tourne sur le Mac de référence.
**Go / no-go** : sur 20 situations réelles, l'action voulue est-elle dans le top 3 au moins 7 fois sur 10 ? Sinon, on revoit l'approche avant d'aller plus loin.

## v0.3 · Décision mesurée (janvier → février)

But : choisir le backend de décision sur des mesures, pas sur une préférence.

- [ ] [Tous] Jeu d'au moins 50 situations réelles, annotées et anonymisées (`lab/datasets/`)
- [ ] [A] Prototype de décision : au moins 2 backends, même jeu, même protocole, mesures côte à côte · **fin janvier** {O1}
- [ ] [A] [D] Protocole de mesure PERF écrit et outillé (latence raccourci → menu) · **fin janvier** {PERF}
- [ ] [A] Décision technique (ADR) : backend retenu, approches écartées, conséquences · **fin février** {O1}
- [ ] [A] [D] Mesure de référence et diagnostic outillé : où ça bloque · **fin février** {PERF}
- [ ] [B] Prototype d'exploration lancé, hypothèse écrite avant · **février → avril** {O1}
- [ ] [A] Prises de contact experts : au moins 6 entre janvier et avril, relances comprises {EXP}

**Démo de fin** : le tableau comparatif des backends, sorti de `make bench`.

## v0.4 · Confiance (mars)

But : rien de risqué ne passe sans l'utilisateur, et on le prouve.

- [ ] [A] Validation qui expire, état du run sérialisable
- [ ] [B] Routage par sensibilité en conditions réelles (fragments `LOCAL_ONLY`)
- [ ] [C] Annulation (`undo`) des actions `REVERSIBLE`
- [ ] [Tous] Jeu adversarial : une injection dans un mail ne déclenche rien sans validation
- [ ] [C] Chemin critique couvert par les tests, taux de couverture publié · **fin mars** {O2}
- [ ] [Tous] Au moins 3 régressions attrapées et documentées · **fin mars** {O2}
- [ ] [D] Test de reproductibilité par une personne extérieure à l'équipe · **fin mars** {O2}
- [ ] [A] [D] Boucle d'optimisation 1 : un changement, une nouvelle mesure, l'écart · **fin mars** {PERF}
- [ ] [A] Premier échange avec un expert (dossier envoyé 48 h avant, compte rendu dans les 48 h) · **fin mars** {EXP}

## v1.0 · Greenlight (avril → mai)

But : la solution retenue est intégrée et mesurée, et chaque objectif a ses preuves.

- [ ] [A] Solution retenue intégrée sur `main`, mesure avant / après sur les critères d'octobre · **fin avril** {O1}
- [ ] [B] Mémoire : `remember()` et personnalisation mesurée (gain du top 1 avec ou sans mémoire)
- [ ] [B] Prototype d'exploration conclu, même négatif · **avril** {O1}
- [ ] [A] [D] Boucle d'optimisation 2 · **fin mai** {PERF}
- [ ] [A] Second échange avec un expert, mémo de fin d'année · **fin mai** {EXP}
- [ ] [A] Rapport qualité final `make quality` · **mai** {O2}
- [ ] [Tous] Au moins 5 contributions publiques, dont une externe · **mai** {O1}
- [ ] [Tous] Répétition du jury : démo scriptée, toutes les preuves de `evidence/` à jour

**Démo de fin** : le jury Greenlight.

## Plus tard (Tech 5, 2027-28)

Rien de tout ça avant la Greenlight.

- mémoire longue qui se restructure seule (la contribution de recherche)
- deuxième surface (le navigateur) : c'est là qu'on extraira les abstractions génériques
- déclencheurs automatiques (planification, webhook), donc des runs qu'on suspend et reprend
- connecteurs MCP
- plus d'actions, catalogue extensible

## Parking

Idées proposées, volontairement pas faites. Format : date · idée · qui · pourquoi pas maintenant.

- 2026-10-15 · moteur de workflow générique (DAG) · A · pas avant deux cas d'usage réels (ADR 0006)
- 2026-10-15 · mobile, AR, voix · Tous · hors scope V1
