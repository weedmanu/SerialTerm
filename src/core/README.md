# Core

Ce module embarque toute la logique métier de l'application.
**Règle absolue** : ce composant ne doit avoir aucune dépendance dirigée vers `gtk4` ou `glib`.

Composants principaux :

- `connection.rs` : Trait d'abstraction asynchrone régissant les états de connexion.
- `serial_manager.rs` : Implémentation I/O pour les ports séries.
- `settings.rs` : Modèles de données des préférences (sauvegardées en JSON).
- `logger.rs` : Journalisation et enrichissement des traces applicatives.
