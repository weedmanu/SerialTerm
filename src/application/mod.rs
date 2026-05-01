//! @file       application/mod.rs
//! @brief      Façade du module `application` — cas d'utilisation métier.
//!
//! Ce module expose la couche **use-cases** (logique applicative pure) :
//! validation des entrées et construction des configurations série.
//!
//! Il ne contient aucune dépendance GTK ni Tokio : sa testabilité est totale.

pub mod use_cases;
