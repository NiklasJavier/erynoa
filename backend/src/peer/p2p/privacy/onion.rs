//! # Onion-Verschlüsselung (RL2-RL4)
//!
//! Implementiert die Schichten-Verschlüsselung für Multi-Hop-Routing.
//!
//! ## Axiom-Referenzen
//!
//! - **RL2**: Wissens-Separation (Informationstheoretisch)
//!   - Jeder Hop kennt nur seinen Vorgänger und Nachfolger
//!   - Kein Hop kann Sender oder Empfänger identifizieren
//!
//! - **RL3**: Schichten-Integrität
//!   - ChaCha20-Poly1305 AEAD für jede Schicht
//!   - Manipulation wird erkannt und abgelehnt
//!
//! - **RL4**: Forward + Backward Secrecy
//!   - Ephemeral X25519 Keys pro Nachricht
//!   - HKDF-basierte Key-Derivation mit Hop-Index
//!
//! ## Core-Logic-Verknüpfungen
//!
//! - **Κ3**: Trust-Vektor der Relays beeinflusst Route-Auswahl (siehe relay_selection)
//! - **Κ4**: Bei Misbehavior schnelle Trust-Reduktion
//!
//! ## Wire-Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    ONION PACKET (Variable)                      │
//! ├──────────┬──────────┬──────────────────────────────────────────┤
//! │ EPK (32) │ Nonce(12)│ Encrypted Layers (Variable)              │
//! │ X25519   │ Random   │ [Layer₁[Layer₂[...[Layerₙ[Payload]]]]]   │
//! └──────────┴──────────┴──────────────────────────────────────────┘
//! ```
//!
//! ## Beispiel
//!
//! ```rust,ignore
//! use erynoa_api::peer::p2p::privacy::onion::{OnionBuilder, OnionDecryptor};
//!
//! // Sender: Onion bauen
//! let builder = OnionBuilder::new(route);
//! let packet = builder.build(b"Hello", &dest_addr);
//!
//! // Relay: Schicht entschlüsseln
//! let mut decryptor = OnionDecryptor::new(relay_private_key);
//! let layer = decryptor.decrypt_layer(&packet)?;
//! // layer.payload weiterleiten an layer.next_relay
//! ```

use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use std::collections::HashSet;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximale Hop-Anzahl (RL7: CRITICAL = 5 + 2 threat)
pub const MAX_HOPS: usize = 7;

/// Minimale Hop-Anzahl (RL2: Wissens-Separation erfordert mindestens 2)
pub const MIN_HOPS: usize = 2;

/// Onion-Layer-Header-Größe (32 Bytes epk + 12 Bytes nonce + 16 Bytes tag)
pub const LAYER_HEADER_SIZE: usize = 60;

/// Nonce-Cache-Größe für Replay-Protection (RL15)
const NONCE_CACHE_SIZE: usize = 10_000;

/// HKDF-Info-Prefix für Key-Derivation
const HKDF_INFO_PREFIX: &str = "erynoa-relay-v1-hop-";

// ============================================================================
// SESSION KEY (Zeroized on Drop)
// ============================================================================

/// Session-Key für einen Hop
///
/// Wird aus X25519 Shared Secret via HKDF abgeleitet.
/// Implementiert Zeroize für sichere Speicherbereinigung.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SessionKey([u8; 32]);

impl SessionKey {
    /// Erstelle SessionKey aus Bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Zugriff auf Key-Bytes (nur intern)
    fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

// ============================================================================
// EPHEMERAL KEY AGREEMENT (RL4)
// ============================================================================

/// Ephemeral Key Agreement (RL4)
///
/// Generiert für jede Nachricht einen neuen X25519 Keypair.
/// Der Secret wird nur vom Sender gehalten, der Public Key
/// wird mit dem Paket übertragen.
///
/// Verwendet `StaticSecret` statt `EphemeralSecret`, da wir mehrere
/// Diffie-Hellman-Operationen durchführen müssen (eine pro Hop).
pub struct EphemeralKeyAgreement {
    /// Secret (nur Sender kennt ihn) - wird nach Build zeroized
    secret: StaticSecret,
    /// Ephemeral Public Key (wird mit jeder Schicht mitgesendet)
    pub public_key: PublicKey,
}

impl EphemeralKeyAgreement {
    /// Erstelle neues Ephemeral-Keypair
    ///
    /// Verwendet den System-CSPRNG für Schlüsselgenerierung.
    pub fn new() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        Self { secret, public_key }
    }

    /// Berechne Session-Key für Relay i (RL4: HKDF-basiert)
    ///
    /// Der Hop-Index wird in die Key-Derivation einbezogen,
    /// um Key-Reuse zwischen verschiedenen Hops zu verhindern.
    ///
    /// ## Formel
    ///
    /// ```text
    /// shared = X25519(ephemeral_secret, relay_public_key)
    /// session_key = HKDF-SHA256(shared, info="erynoa-relay-v1-hop-{i}")
    /// ```
    pub fn derive_session_key(&self, relay_public_key: &PublicKey, hop_index: u8) -> SessionKey {
        let shared = self.secret.diffie_hellman(relay_public_key);

        // HKDF mit Hop-Index als Info (verhindert Key-Reuse)
        let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
        let info = format!("{}{}", HKDF_INFO_PREFIX, hop_index);

        let mut key = [0u8; 32];
        hk.expand(info.as_bytes(), &mut key)
            .expect("HKDF expand failed - should never happen with 32-byte output");

        SessionKey(key)
    }
}

impl Default for EphemeralKeyAgreement {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ONION LAYER
// ============================================================================

/// Einzelne Onion-Schicht (für Debugging/Testing)
#[derive(Debug, Clone)]
pub struct OnionLayer {
    /// Verschlüsselter Payload (inkl. nächste Schicht oder Klartext)
    pub ciphertext: Vec<u8>,
    /// Ephemeral Public Key für diesen Hop
    pub ephemeral_pk: [u8; 32],
    /// Nonce für AEAD
    pub nonce: [u8; 12],
}

// ============================================================================
// ONION BUILDER (Sender-Seite)
// ============================================================================

/// Onion-Paket-Builder
///
/// Konstruiert Onion-verschlüsselte Pakete von innen nach außen.
/// Jede Schicht wird mit dem Session-Key des entsprechenden Relays
/// verschlüsselt.
///
/// ## Beispiel
///
/// ```rust,ignore
/// let route = vec![relay1_pk, relay2_pk, relay3_pk];
/// let builder = OnionBuilder::new(route);
/// let packet = builder.build(b"Secret message", &destination);
/// ```
pub struct OnionBuilder {
    /// Route: [Ingress, Middle..., Egress]
    route: Vec<PublicKey>,
    /// Ephemeral Key Agreement
    key_agreement: EphemeralKeyAgreement,
}

impl OnionBuilder {
    /// Erstelle neuen OnionBuilder für die gegebene Route
    ///
    /// ## Panics
    ///
    /// Panikt wenn:
    /// - Route weniger als MIN_HOPS (2) hat
    /// - Route mehr als MAX_HOPS (7) hat
    pub fn new(route: Vec<PublicKey>) -> Self {
        assert!(
            route.len() >= MIN_HOPS,
            "Minimum {} Hops required (RL2: Wissens-Separation)",
            MIN_HOPS
        );
        assert!(
            route.len() <= MAX_HOPS,
            "Maximum {} Hops allowed (Performance)",
            MAX_HOPS
        );

        Self {
            route,
            key_agreement: EphemeralKeyAgreement::new(),
        }
    }

    /// Erstelle mit explizitem Key-Agreement (für Testing)
    #[cfg(test)]
    pub fn with_key_agreement(route: Vec<PublicKey>, key_agreement: EphemeralKeyAgreement) -> Self {
        assert!(route.len() >= MIN_HOPS && route.len() <= MAX_HOPS);
        Self {
            route,
            key_agreement,
        }
    }

    /// Baue Onion-Paket (von innen nach außen) – RL3
    ///
    /// ## Formel (RL2)
    ///
    /// ```text
    /// Ω(M, π) = E_{K₁}(E_{K₂}(...E_{Kₙ}(M || addr(dest))...|| addr(R₃)) || addr(R₂))
    /// ```
    ///
    /// ## Wire-Format (Ergebnis)
    ///
    /// ```text
    /// [EPK:32][Nonce:12][Encrypted Layer_1[Layer_2[...[Payload || Dest]]]]
    /// ```
    ///
    /// ## Arguments
    ///
    /// - `plaintext`: Der zu übertragende Klartext
    /// - `dest_addr`: Ziel-Adresse (32 Bytes, typischerweise PublicKey)
    ///
    /// ## Returns
    ///
    /// Vollständiges Onion-Paket als Byte-Vektor
    pub fn build(&self, plaintext: &[u8], dest_addr: &[u8]) -> Vec<u8> {
        assert_eq!(dest_addr.len(), 32, "Destination address must be 32 bytes");

        let mut payload = Vec::with_capacity(plaintext.len() + dest_addr.len() + 2);

        // Innerste Schicht: Plaintext + Ziel-Adresse
        payload.extend_from_slice(plaintext);
        payload.extend_from_slice(dest_addr);

        // Von innen (Egress) nach außen (Ingress) verschlüsseln
        for (i, relay_pk) in self.route.iter().rev().enumerate() {
            let hop_index = (self.route.len() - 1 - i) as u8;
            let session_key = self.key_agreement.derive_session_key(relay_pk, hop_index);

            // Nächste Relay-Adresse hinzufügen (außer für innerste Schicht)
            if i > 0 {
                let next_relay_addr = self.route[self.route.len() - i].as_bytes();
                payload.extend_from_slice(next_relay_addr);
            }

            payload = self.encrypt_layer(&session_key, &payload, hop_index);
        }

        // Ephemeral Public Key voranstellen
        let mut packet = Vec::with_capacity(32 + payload.len());
        packet.extend_from_slice(self.key_agreement.public_key.as_bytes());
        packet.extend(payload);

        packet
    }

    /// Verschlüssele eine Schicht mit ChaCha20-Poly1305
    ///
    /// ## Wire-Format (pro Schicht)
    ///
    /// ```text
    /// [Nonce:12][Ciphertext:N][Tag:16]
    /// ```
    fn encrypt_layer(&self, key: &SessionKey, plaintext: &[u8], hop_index: u8) -> Vec<u8> {
        let cipher =
            ChaCha20Poly1305::new_from_slice(key.as_bytes()).expect("Invalid key length - bug");

        // Nonce: 8 random bytes + 4 bytes hop_index (für Replay-Schutz)
        let mut nonce = [0u8; 12];
        getrandom::getrandom(&mut nonce[..8]).expect("CSPRNG failed - system error");
        nonce[8..12].copy_from_slice(&(hop_index as u32).to_le_bytes());

        let ciphertext = cipher
            .encrypt(nonce.as_ref().into(), plaintext)
            .expect("Encryption failed - should never happen with valid key");

        // Nonce + Ciphertext (Tag ist in Ciphertext enthalten)
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend(ciphertext);

        result
    }

    /// Hole die Route-Länge
    pub fn hop_count(&self) -> usize {
        self.route.len()
    }
}

// ============================================================================
// ONION DECRYPTOR (Relay-Seite)
// ============================================================================

/// Onion-Layer-Entschlüsselung (für Relay-Nodes)
///
/// Entschlüsselt eine Schicht des Onion-Pakets und extrahiert:
/// - Die nächste Relay-Adresse (oder Ziel)
/// - Den inneren Payload (nächste Schicht oder Klartext)
///
/// ## Replay-Protection (RL15)
///
/// Verwendet einen Nonce-Cache um Replay-Attacken zu verhindern.
/// Jede gesehene Nonce wird gespeichert und bei Wiederholung abgelehnt.
pub struct OnionDecryptor {
    /// Private Key des Relays
    private_key: StaticSecret,
    /// Replay-Protection: Nonce-Cache (RL15)
    nonce_cache: HashSet<[u8; 12]>,
    /// Public Key (für Debugging)
    #[allow(dead_code)]
    public_key: PublicKey,
}

impl OnionDecryptor {
    /// Erstelle neuen Decryptor für Relay-Private-Key
    pub fn new(private_key: StaticSecret) -> Self {
        let public_key = PublicKey::from(&private_key);
        Self {
            private_key,
            nonce_cache: HashSet::with_capacity(NONCE_CACHE_SIZE),
            public_key,
        }
    }

    /// Entschlüssele eine Schicht – RL3
    ///
    /// ## Formel
    ///
    /// ```text
    /// D_{Kᵢ}(Layer_i) = Layer_{i+1} || addr(R_{i+1})
    /// ```
    ///
    /// ## Wire-Format (Input)
    ///
    /// ```text
    /// [EPK:32][Nonce:12][Ciphertext:N][Tag:16]
    /// ```
    ///
    /// ## Returns
    ///
    /// - `Ok(DecryptedLayer)`: Erfolgreich entschlüsselt
    /// - `Err(OnionError)`: Entschlüsselung fehlgeschlagen
    pub fn decrypt_layer(&mut self, packet: &[u8]) -> Result<DecryptedLayer, OnionError> {
        // Minimum: 32 (EPK) + 12 (Nonce) + 16 (Tag) + 32 (min payload) = 92 bytes
        if packet.len() < 32 + 12 + 16 + 32 {
            return Err(OnionError::PacketTooSmall {
                actual: packet.len(),
                minimum: 92,
            });
        }

        // Ephemeral Public Key extrahieren
        let epk_bytes: [u8; 32] = packet[..32]
            .try_into()
            .map_err(|_| OnionError::InvalidEphemeralKey)?;
        let ephemeral_pk = PublicKey::from(epk_bytes);

        // Nonce extrahieren und Replay prüfen (RL15)
        let nonce: [u8; 12] = packet[32..44]
            .try_into()
            .map_err(|_| OnionError::InvalidNonce)?;

        if self.nonce_cache.contains(&nonce) {
            return Err(OnionError::ReplayDetected);
        }

        // Hop-Index aus Nonce extrahieren
        let hop_index = u32::from_le_bytes(
            nonce[8..12]
                .try_into()
                .map_err(|_| OnionError::InvalidNonce)?,
        ) as u8;

        // Shared Secret berechnen
        let shared = self.private_key.diffie_hellman(&ephemeral_pk);

        // Session Key ableiten
        let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
        let info = format!("{}{}", HKDF_INFO_PREFIX, hop_index);
        let mut key = [0u8; 32];
        hk.expand(info.as_bytes(), &mut key)
            .map_err(|_| OnionError::KeyDerivationFailed)?;

        // Entschlüsseln
        let cipher =
            ChaCha20Poly1305::new_from_slice(&key).map_err(|_| OnionError::InvalidKey)?;

        let ciphertext = &packet[44..];
        let plaintext = cipher
            .decrypt(nonce.as_ref().into(), ciphertext)
            .map_err(|_| OnionError::DecryptionFailed)?;

        // Nonce in Cache speichern (nach erfolgreicher Entschlüsselung)
        self.add_nonce_to_cache(nonce);

        // Parse: Payload + nächste Relay-Adresse (letzte 32 Bytes)
        if plaintext.len() < 32 {
            return Err(OnionError::InvalidPayload {
                reason: "Payload too small to contain next address",
            });
        }

        let next_relay_bytes: [u8; 32] = plaintext[plaintext.len() - 32..]
            .try_into()
            .map_err(|_| OnionError::InvalidPayload {
                reason: "Failed to extract next relay address",
            })?;
        let next_relay = PublicKey::from(next_relay_bytes);
        let inner_payload = plaintext[..plaintext.len() - 32].to_vec();

        // Prüfe ob dies die letzte Schicht ist
        // Heuristik: Wenn inner_payload zu klein für weitere Schicht ist
        let is_final = inner_payload.len() < 44; // Nonce(12) + Tag(16) + min_content

        Ok(DecryptedLayer {
            next_relay,
            payload: inner_payload,
            is_final,
            hop_index,
        })
    }

    /// Füge Nonce zum Cache hinzu (mit Size-Limit)
    fn add_nonce_to_cache(&mut self, nonce: [u8; 12]) {
        // Bei Überlauf: Cache leeren (TODO: LRU in Production)
        if self.nonce_cache.len() >= NONCE_CACHE_SIZE {
            self.nonce_cache.clear();
        }
        self.nonce_cache.insert(nonce);
    }

    /// Lösche Nonce-Cache (für Testing)
    #[cfg(test)]
    pub fn clear_nonce_cache(&mut self) {
        self.nonce_cache.clear();
    }

    /// Anzahl gecachter Nonces
    pub fn nonce_cache_size(&self) -> usize {
        self.nonce_cache.len()
    }
}

// ============================================================================
// DECRYPTED LAYER RESULT
// ============================================================================

/// Entschlüsseltes Layer-Ergebnis
#[derive(Debug, Clone)]
pub struct DecryptedLayer {
    /// Nächster Relay (oder Ziel wenn is_final)
    pub next_relay: PublicKey,
    /// Payload (nächste Schicht oder Klartext)
    pub payload: Vec<u8>,
    /// Ist dies die letzte Schicht? (Egress)
    pub is_final: bool,
    /// Hop-Index (aus Nonce extrahiert)
    pub hop_index: u8,
}

impl DecryptedLayer {
    /// Hole nächste Relay-Adresse als Bytes
    pub fn next_relay_bytes(&self) -> [u8; 32] {
        *self.next_relay.as_bytes()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Fehler bei Onion-Operationen
#[derive(Debug, thiserror::Error)]
pub enum OnionError {
    #[error("Packet too small: {actual} bytes, minimum {minimum} required")]
    PacketTooSmall { actual: usize, minimum: usize },

    #[error("Invalid ephemeral public key")]
    InvalidEphemeralKey,

    #[error("Invalid nonce format")]
    InvalidNonce,

    #[error("Replay attack detected (RL15)")]
    ReplayDetected,

    #[error("Key derivation failed")]
    KeyDerivationFailed,

    #[error("Invalid key")]
    InvalidKey,

    #[error("Decryption failed - integrity violation (RL3)")]
    DecryptionFailed,

    #[error("Invalid payload: {reason}")]
    InvalidPayload { reason: &'static str },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    /// Generiere Relay-Keypairs für Tests
    fn generate_relay_keypairs(count: usize) -> Vec<(StaticSecret, PublicKey)> {
        (0..count)
            .map(|_| {
                let secret = StaticSecret::random_from_rng(OsRng);
                let public = PublicKey::from(&secret);
                (secret, public)
            })
            .collect()
    }

    #[test]
    fn test_onion_roundtrip_3_hops() {
        // 3 Relays generieren
        let keypairs = generate_relay_keypairs(3);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        // Onion bauen
        let builder = OnionBuilder::new(route.clone());
        let plaintext = b"Hello, World!";
        let dest_addr = [42u8; 32]; // Dummy-Ziel-Adresse
        let packet = builder.build(plaintext, &dest_addr);

        // Schicht für Schicht entschlüsseln
        let mut current_packet = packet;
        for (i, (secret, _)) in keypairs.iter().enumerate() {
            let mut decryptor = OnionDecryptor::new(secret.clone());
            let layer = decryptor.decrypt_layer(&current_packet).unwrap();

            // Prüfe Hop-Index
            assert_eq!(layer.hop_index, i as u8);

            if i < keypairs.len() - 1 {
                // Mittlere Hops
                assert!(!layer.is_final, "Hop {} sollte nicht final sein", i);
                assert_eq!(
                    layer.next_relay_bytes(),
                    *route[i + 1].as_bytes(),
                    "Falsche nächste Relay-Adresse bei Hop {}",
                    i
                );

                // Innerer Payload für nächste Iteration
                // Wir müssen die EPK neu voranstellen für die nächste Iteration
                let mut next_packet = Vec::with_capacity(32 + layer.payload.len());
                next_packet.extend_from_slice(current_packet[..32].as_ref()); // EPK wiederverwenden
                next_packet.extend_from_slice(&layer.payload);
                current_packet = next_packet;
            } else {
                // Letzte Schicht (Egress)
                assert!(layer.is_final, "Letzte Schicht sollte final sein");

                // Prüfe Ziel-Adresse
                assert_eq!(
                    layer.next_relay_bytes(),
                    dest_addr,
                    "Ziel-Adresse stimmt nicht"
                );

                // Prüfe Plaintext
                assert_eq!(
                    &layer.payload[..plaintext.len()],
                    plaintext,
                    "Plaintext stimmt nicht"
                );
            }
        }
    }

    #[test]
    fn test_replay_protection() {
        let keypairs = generate_relay_keypairs(2);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        let builder = OnionBuilder::new(route);
        let packet = builder.build(b"Test", &[0u8; 32]);

        let mut decryptor = OnionDecryptor::new(keypairs[0].0.clone());

        // Erstes Entschlüsseln funktioniert
        let result1 = decryptor.decrypt_layer(&packet);
        assert!(result1.is_ok());

        // Zweites Entschlüsseln (Replay) schlägt fehl
        let result2 = decryptor.decrypt_layer(&packet);
        assert!(matches!(result2, Err(OnionError::ReplayDetected)));
    }

    #[test]
    fn test_minimum_hops() {
        let keypairs = generate_relay_keypairs(2);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        // Sollte funktionieren mit 2 Hops
        let builder = OnionBuilder::new(route);
        assert_eq!(builder.hop_count(), 2);
    }

    #[test]
    #[should_panic(expected = "Minimum")]
    fn test_too_few_hops() {
        let keypairs = generate_relay_keypairs(1);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        // Sollte paniken mit nur 1 Hop
        let _builder = OnionBuilder::new(route);
    }

    #[test]
    #[should_panic(expected = "Maximum")]
    fn test_too_many_hops() {
        let keypairs = generate_relay_keypairs(8);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        // Sollte paniken mit 8 Hops (max ist 7)
        let _builder = OnionBuilder::new(route);
    }

    #[test]
    fn test_packet_too_small() {
        let keypairs = generate_relay_keypairs(2);
        let mut decryptor = OnionDecryptor::new(keypairs[0].0.clone());

        // Zu kleines Paket
        let small_packet = vec![0u8; 50];
        let result = decryptor.decrypt_layer(&small_packet);

        assert!(matches!(
            result,
            Err(OnionError::PacketTooSmall { actual: 50, .. })
        ));
    }

    #[test]
    fn test_invalid_ciphertext() {
        let keypairs = generate_relay_keypairs(2);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        let builder = OnionBuilder::new(route);
        let mut packet = builder.build(b"Test", &[0u8; 32]);

        // Ciphertext korrumpieren
        packet[50] ^= 0xFF;

        let mut decryptor = OnionDecryptor::new(keypairs[0].0.clone());
        let result = decryptor.decrypt_layer(&packet);

        assert!(matches!(result, Err(OnionError::DecryptionFailed)));
    }

    #[test]
    fn test_session_key_zeroize() {
        let key = SessionKey::from_bytes([1u8; 32]);
        assert_eq!(key.as_bytes(), &[1u8; 32]);
        // Nach Drop sollte der Key zeroized werden (nicht direkt testbar,
        // aber der Zeroize-Macro garantiert es)
        drop(key);
    }

    #[test]
    fn test_max_hops_roundtrip() {
        // Test mit maximaler Hop-Anzahl
        let keypairs = generate_relay_keypairs(MAX_HOPS);
        let route: Vec<PublicKey> = keypairs.iter().map(|(_, pk)| *pk).collect();

        let builder = OnionBuilder::new(route);
        let packet = builder.build(b"Max hops test", &[99u8; 32]);

        // Erstes Relay sollte entschlüsseln können
        let mut decryptor = OnionDecryptor::new(keypairs[0].0.clone());
        let result = decryptor.decrypt_layer(&packet);
        assert!(result.is_ok());
    }
}
