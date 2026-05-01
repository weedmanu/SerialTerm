// =============================================================================
// Fichier : connection.rs
// Rôle    : Trait d'abstraction pour les connexions série
//
// Principe SOLID :
//   - Le core ne dépend d'aucun toolkit UI (pas de glib/gtk ici).
//   - Le pont UI↔core se fait dans window.rs via async_channel.
// =============================================================================

use anyhow::Result;
use async_trait::async_trait;

/// Type de connexion supporté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Serial,
}

/// État de la connexion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Déconnecté"),
            Self::Connecting => write!(f, "Connexion..."),
            Self::Connected => write!(f, "Connecté"),
            Self::Error => write!(f, "Erreur"),
        }
    }
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial => write!(f, "Série"),
        }
    }
}

/// Événements envoyés par la connexion vers l'UI.
///
/// SOLID : ce type n'a aucune dépendance vers GTK/glib.
#[derive(Debug)]
pub enum ConnectionEvent {
    /// Connexion établie avec succès.
    Connected {
        conn_type: ConnectionType,
        description: String,
    },
    /// Données reçues du périphérique distant.
    DataReceived(Vec<u8>),
    /// Connexion fermée proprement.
    Disconnected,
    /// Erreur non-récupérable (affichée dans le terminal).
    ///
    /// Note : quand `connect()` échoue, seul cet événement est émis.
    /// `Disconnected` n'est PAS envoyé ensuite — l'UI doit traiter `Error`
    /// comme une fin de session à part entière.
    Error(String),
}

/// Commandes envoyées par l'UI vers la connexion.
#[derive(Debug)]
pub enum ConnectionCommand {
    SendData(Vec<u8>),
    ResizeTerminal {
        columns: u32,
        rows: u32,
        pixel_width: u32,
        pixel_height: u32,
    },
    Disconnect,
}

/// Trait unifié pour toutes les connexions.
///
/// Permet de manipuler les connexions série de manière polymorphique.
/// SOLID : aucune dépendance UI dans ce trait.
#[async_trait]
pub trait Connection: Send {
    /// Établit la connexion.
    async fn connect(&mut self) -> Result<()>;

    /// Ferme proprement la connexion.
    async fn disconnect(&mut self) -> Result<()>;

    /// Envoie des données brutes.
    async fn send(&mut self, data: &[u8]) -> Result<usize>;

    /// Lit les données disponibles (non-bloquant).
    /// Retourne les octets lus, ou un vecteur vide si rien n'est disponible.
    async fn read(&mut self) -> Result<Vec<u8>>;

    /// Redimensionne le terminal distant quand le viewport local change.
    ///
    /// Implémentation par défaut : no-op, utile pour la connexion série.
    async fn resize_terminal(
        &mut self,
        _columns: u32,
        _rows: u32,
        _pixel_width: u32,
        _pixel_height: u32,
    ) -> Result<()> {
        Ok(())
    }

    /// Retourne l'état courant de la connexion.
    fn state(&self) -> ConnectionState;

    /// Retourne le type de connexion.
    fn connection_type(&self) -> ConnectionType;

    /// Retourne une description de la connexion (ex: "COM3 @ 115200").
    fn description(&self) -> String;

    /// Retourne le nombre d'octets envoyés depuis la connexion.
    fn bytes_sent(&self) -> u64;

    /// Retourne le nombre d'octets reçus depuis la connexion.
    fn bytes_received(&self) -> u64;
}

/// Lance une tâche asynchrone pour gérer la connexion.
///
/// # Architecture
/// - Entrée (UI → core) : `tokio::sync::mpsc::Sender<ConnectionCommand>`
/// - Sortie (core → UI) : `async_channel::Receiver<ConnectionEvent>`
///
/// Le core ne dépend d'aucun toolkit UI. Le pont vers `GLib` est dans window.rs.
pub fn spawn_connection_actor(
    mut connection: Box<dyn Connection>,
) -> (
    tokio::sync::mpsc::Sender<ConnectionCommand>,
    async_channel::Receiver<ConnectionEvent>,
) {
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<ConnectionCommand>(32);
    // bounded(128) : backpressure si l'UI consomme trop lentement
    let (event_tx, event_rx) = async_channel::bounded::<ConnectionEvent>(128);

    tokio::spawn(async move {
        // ── Phase 1 : Connexion ────────────────────────────────────────────────
        // La connexion se fait dans la tâche tokio, libérant le thread GTK.
        match connection.connect().await {
            Ok(()) => {
                let _ = event_tx
                    .send(ConnectionEvent::Connected {
                        conn_type: connection.connection_type(),
                        description: connection.description(),
                    })
                    .await;
            }
            Err(e) => {
                let _ = event_tx.send(ConnectionEvent::Error(e.to_string())).await;
                return; // N'entre pas dans la boucle I/O
            }
        }

        // ── Phase 2 : Boucle I/O ──────────────────────────────────────────────
        loop {
            tokio::select! {
                biased; // prioritise les commandes UI sur la lecture

                // Commandes depuis l'UI
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(ConnectionCommand::SendData(data)) => {
                            if let Err(e) = connection.send(&data).await {
                                let _ = connection.disconnect().await;
                                let _ = event_tx.send(ConnectionEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                        Some(ConnectionCommand::ResizeTerminal {
                            columns,
                            rows,
                            pixel_width,
                            pixel_height,
                        }) => {
                            if let Err(e) = connection
                                .resize_terminal(columns, rows, pixel_width, pixel_height)
                                .await
                            {
                                let _ = connection.disconnect().await;
                                let _ = event_tx.send(ConnectionEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                        Some(ConnectionCommand::Disconnect) | None => {
                            // Déconnexion propre demandée ou channel fermé
                            let _ = connection.disconnect().await;
                            let _ = event_tx.send(ConnectionEvent::Disconnected).await;
                            break;
                        }
                    }
                }

                // Lecture depuis la connexion
                read_result = connection.read() => {
                    match read_result {
                        Ok(data) if !data.is_empty() => {
                            if event_tx.send(ConnectionEvent::DataReceived(data)).await.is_err() {
                                // L'UI ne consomme plus → on arrête
                                let _ = connection.disconnect().await;
                                break;
                            }
                        }
                        Ok(_) => {
                            // Pas de données ; vérifier déconnexion spontanée
                            let s = connection.state();
                            if s == ConnectionState::Disconnected || s == ConnectionState::Error {
                                // Fermer proprement quand la connexion remonte une fin de flux.
                                let _ = connection.disconnect().await;
                                let _ = event_tx.send(ConnectionEvent::Disconnected).await;
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = connection.disconnect().await;
                            let _ = event_tx.send(ConnectionEvent::Error(e.to_string())).await;
                            break;
                        }
                    }
                }
            }
        }

        log::info!(
            "Connexion terminée — envoyés: {} octets, reçus: {} octets",
            connection.bytes_sent(),
            connection.bytes_received()
        );
        log::debug!("Acteur de connexion arrêté proprement.");
    });

    (cmd_tx, event_rx)
}

// =============================================================================
// Tests unitaires
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::timeout;

    #[derive(Clone, Debug)]
    enum ReadStep {
        Data(Vec<u8>),
        Empty(ConnectionState),
        Error(&'static str),
    }

    #[derive(Debug)]
    struct MockState {
        connection_type: ConnectionType,
        description: String,
        state: ConnectionState,
        connect_calls: usize,
        disconnect_calls: usize,
        sent_payloads: Vec<Vec<u8>>,
        read_steps: VecDeque<ReadStep>,
        connect_error: Option<&'static str>,
        send_error: Option<&'static str>,
        bytes_sent: u64,
        bytes_received: u64,
    }

    impl Default for MockState {
        fn default() -> Self {
            Self {
                connection_type: ConnectionType::Serial,
                description: "mock connection".to_string(),
                state: ConnectionState::Disconnected,
                connect_calls: 0,
                disconnect_calls: 0,
                sent_payloads: Vec::new(),
                read_steps: VecDeque::new(),
                connect_error: None,
                send_error: None,
                bytes_sent: 0,
                bytes_received: 0,
            }
        }
    }

    struct MockConnection {
        shared: Arc<Mutex<MockState>>,
    }

    impl MockConnection {
        fn new(shared: Arc<Mutex<MockState>>) -> Self {
            Self { shared }
        }
    }

    #[async_trait]
    impl Connection for MockConnection {
        async fn connect(&mut self) -> Result<()> {
            let mut shared = self.shared.lock().expect("mock state poisoned");
            shared.connect_calls = shared.connect_calls.saturating_add(1);
            if let Some(error) = shared.connect_error.take() {
                return Err(anyhow!(error));
            }
            shared.state = ConnectionState::Connected;
            drop(shared);
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            let mut shared = self.shared.lock().expect("mock state poisoned");
            shared.disconnect_calls = shared.disconnect_calls.saturating_add(1);
            shared.state = ConnectionState::Disconnected;
            drop(shared);
            Ok(())
        }

        async fn send(&mut self, data: &[u8]) -> Result<usize> {
            let mut shared = self.shared.lock().expect("mock state poisoned");
            if let Some(error) = shared.send_error.take() {
                return Err(anyhow!(error));
            }
            shared.sent_payloads.push(data.to_vec());
            shared.bytes_sent = shared
                .bytes_sent
                .saturating_add(u64::try_from(data.len()).unwrap_or(0));
            drop(shared);
            Ok(data.len())
        }

        async fn read(&mut self) -> Result<Vec<u8>> {
            let step = {
                self.shared
                    .lock()
                    .expect("mock state poisoned")
                    .read_steps
                    .pop_front()
            };

            match step {
                Some(ReadStep::Data(data)) => {
                    let mut shared = self.shared.lock().expect("mock state poisoned");
                    shared.bytes_received = shared
                        .bytes_received
                        .saturating_add(u64::try_from(data.len()).unwrap_or(0));
                    drop(shared);
                    Ok(data)
                }
                Some(ReadStep::Empty(next_state)) => {
                    self.shared.lock().expect("mock state poisoned").state = next_state;
                    Ok(Vec::new())
                }
                Some(ReadStep::Error(message)) => {
                    self.shared.lock().expect("mock state poisoned").state = ConnectionState::Error;
                    Err(anyhow!(message))
                }
                None => {
                    tokio::task::yield_now().await;
                    Ok(Vec::new())
                }
            }
        }

        fn state(&self) -> ConnectionState {
            self.shared.lock().expect("mock state poisoned").state
        }

        fn connection_type(&self) -> ConnectionType {
            self.shared
                .lock()
                .expect("mock state poisoned")
                .connection_type
        }

        fn description(&self) -> String {
            self.shared
                .lock()
                .expect("mock state poisoned")
                .description
                .clone()
        }

        fn bytes_sent(&self) -> u64 {
            self.shared.lock().expect("mock state poisoned").bytes_sent
        }

        fn bytes_received(&self) -> u64 {
            self.shared
                .lock()
                .expect("mock state poisoned")
                .bytes_received
        }
    }

    async fn recv_event(event_rx: &async_channel::Receiver<ConnectionEvent>) -> ConnectionEvent {
        timeout(Duration::from_secs(1), event_rx.recv())
            .await
            .expect("timed out waiting for connection event")
            .expect("event channel unexpectedly closed")
    }

    async fn settle_actor() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // ── ConnectionState display ───────────────────────────────────────────────

    #[test]
    fn state_display_disconnected() {
        assert_eq!(ConnectionState::Disconnected.to_string(), "Déconnecté");
    }

    #[test]
    fn state_display_connecting() {
        assert_eq!(ConnectionState::Connecting.to_string(), "Connexion...");
    }

    #[test]
    fn state_display_connected() {
        assert_eq!(ConnectionState::Connected.to_string(), "Connecté");
    }

    #[test]
    fn state_display_error() {
        assert_eq!(ConnectionState::Error.to_string(), "Erreur");
    }

    // ── ConnectionType display ────────────────────────────────────────────────

    #[test]
    fn type_display_serial() {
        assert_eq!(ConnectionType::Serial.to_string(), "Série");
    }

    // ── ConnectionState equality ──────────────────────────────────────────────

    #[test]
    fn state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Error);
    }

    #[test]
    fn type_equality() {
        assert_eq!(ConnectionType::Serial, ConnectionType::Serial);
    }

    // ── ConnectionState copy ──────────────────────────────────────────────────

    #[test]
    fn state_is_copy() {
        let s = ConnectionState::Connected;
        let s2 = s; // would fail to compile if not Copy
        assert_eq!(s, s2);
    }

    #[test]
    fn type_is_copy() {
        let t = ConnectionType::Serial;
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[tokio::test]
    async fn spawn_connection_actor_emits_connected_then_disconnects_on_command() {
        let shared = Arc::new(Mutex::new(MockState::default()));
        let (cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        match recv_event(&event_rx).await {
            ConnectionEvent::Connected {
                conn_type,
                description,
            } => {
                assert_eq!(conn_type, ConnectionType::Serial);
                assert_eq!(description, "mock connection");
            }
            other => panic!("unexpected event: {other:?}"),
        }

        cmd_tx
            .send(ConnectionCommand::Disconnect)
            .await
            .expect("disconnect command should be sent");

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Disconnected
        ));

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.connect_calls, 1);
        assert_eq!(shared.disconnect_calls, 1);
        drop(shared);
    }

    #[tokio::test]
    async fn spawn_connection_actor_emits_error_when_connect_fails() {
        let shared = Arc::new(Mutex::new(MockState {
            connect_error: Some("connect boom"),
            ..MockState::default()
        }));
        let (_cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        match recv_event(&event_rx).await {
            ConnectionEvent::Error(message) => assert!(message.contains("connect boom")),
            other => panic!("unexpected event: {other:?}"),
        }

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.connect_calls, 1);
        assert_eq!(shared.disconnect_calls, 0);
        drop(shared);
    }

    #[tokio::test]
    async fn spawn_connection_actor_forwards_send_command_payload() {
        let shared = Arc::new(Mutex::new(MockState::default()));
        let (cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Connected { .. }
        ));

        cmd_tx
            .send(ConnectionCommand::SendData(b"hello".to_vec()))
            .await
            .expect("send command should be sent");
        cmd_tx
            .send(ConnectionCommand::Disconnect)
            .await
            .expect("disconnect command should be sent");

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Disconnected
        ));

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.sent_payloads, vec![b"hello".to_vec()]);
        assert_eq!(shared.bytes_sent, 5);
        drop(shared);
    }

    #[tokio::test]
    async fn spawn_connection_actor_emits_data_then_disconnects_when_peer_closes() {
        let shared = Arc::new(Mutex::new(MockState {
            read_steps: VecDeque::from([
                ReadStep::Data(vec![1, 2, 3]),
                ReadStep::Empty(ConnectionState::Disconnected),
            ]),
            ..MockState::default()
        }));
        let (_cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Connected { .. }
        ));

        match recv_event(&event_rx).await {
            ConnectionEvent::DataReceived(data) => assert_eq!(data, vec![1, 2, 3]),
            other => panic!("unexpected event: {other:?}"),
        }

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Disconnected
        ));

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.bytes_received, 3);
        assert_eq!(shared.disconnect_calls, 1);
        drop(shared);
    }

    #[tokio::test]
    async fn spawn_connection_actor_emits_error_when_send_fails() {
        let shared = Arc::new(Mutex::new(MockState {
            send_error: Some("send boom"),
            ..MockState::default()
        }));
        let (cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Connected { .. }
        ));

        cmd_tx
            .send(ConnectionCommand::SendData(b"hello".to_vec()))
            .await
            .expect("send command should be sent");

        match recv_event(&event_rx).await {
            ConnectionEvent::Error(message) => assert!(message.contains("send boom")),
            other => panic!("unexpected event: {other:?}"),
        }

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.disconnect_calls, 1);
        drop(shared);
    }

    #[tokio::test]
    async fn spawn_connection_actor_emits_error_when_read_fails() {
        let shared = Arc::new(Mutex::new(MockState {
            read_steps: VecDeque::from([ReadStep::Error("read boom")]),
            ..MockState::default()
        }));
        let (_cmd_tx, event_rx) =
            spawn_connection_actor(Box::new(MockConnection::new(shared.clone())));

        assert!(matches!(
            recv_event(&event_rx).await,
            ConnectionEvent::Connected { .. }
        ));

        match recv_event(&event_rx).await {
            ConnectionEvent::Error(message) => assert!(message.contains("read boom")),
            other => panic!("unexpected event: {other:?}"),
        }

        settle_actor().await;

        let shared = shared.lock().expect("mock state poisoned");
        assert_eq!(shared.disconnect_calls, 1);
        assert_eq!(shared.state, ConnectionState::Disconnected);
        drop(shared);
    }
}
