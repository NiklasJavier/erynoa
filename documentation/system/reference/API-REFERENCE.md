# Erynoa API Referenz – gRPC/Connect-RPC

> **Version:** 1.0.0
> **Datum:** Februar 2026
> **Status:** Production-Ready
> **Protokoll:** Connect-RPC (gRPC-Web kompatibel)

---

## Executive Summary

Die Erynoa API verwendet **Connect-RPC** als primäres Kommunikationsprotokoll. Connect-RPC bietet:

- **Typsicherheit**: Protobuf-Schema als Single Source of Truth
- **Performance**: Binary Encoding für effiziente Übertragung
- **Browser-Kompatibilität**: gRPC-Web Support ohne Proxy
- **Streaming**: Bidirektionales Streaming für Echtzeit-Updates

---

## I. Protokoll & Endpoints

### 1.1 Base URLs

| Umgebung    | URL                                      |
| ----------- | ---------------------------------------- |
| Development | `http://localhost:3000/api/v1/connect/`  |
| Production  | `https://api.erynoa.com/api/v1/connect/` |

### 1.2 Protokoll-Format

```
POST /api/v1/connect/{package}.{Service}/{Method}
Content-Type: application/proto
```

**Beispiel:**

```
POST /api/v1/connect/erynoa.v1.HealthService/Check
```

### 1.3 REST-Fallback (Nur Health/Info)

Für Load Balancer und Kubernetes-Probes existieren REST-Fallbacks:

| Endpoint             | Beschreibung    |
| -------------------- | --------------- |
| `GET /api/v1/health` | Liveness Probe  |
| `GET /api/v1/ready`  | Readiness Probe |
| `GET /api/v1/info`   | API-Info        |
| `GET /api/v1/status` | Service-Status  |

---

## II. Services

### 2.1 HealthService

Health-Checks für Liveness und Readiness Probes.

```protobuf
service HealthService {
  rpc Check(CheckRequest) returns (CheckResponse);
  rpc Ready(ReadyRequest) returns (ReadyResponse);
}
```

#### Check

Gibt den Liveness-Status zurück.

**Request:** `CheckRequest` (leer)

**Response:**

```protobuf
message CheckResponse {
  enum ServingStatus {
    SERVING_STATUS_UNSPECIFIED = 0;
    SERVING_STATUS_SERVING = 1;
    SERVING_STATUS_NOT_SERVING = 2;
  }
  ServingStatus status = 1;
}
```

#### Ready

Gibt detaillierten Readiness-Status mit Dependency-Checks zurück.

**Request:** `ReadyRequest` (leer)

**Response:**

```protobuf
message ReadyResponse {
  bool ready = 1;
  ServiceStatus database = 2;
  ServiceStatus cache = 3;
  ServiceStatus auth = 4;
  ServiceStatus storage = 5;
}

message ServiceStatus {
  bool healthy = 1;
  string message = 2;
  int64 latency_ms = 3;
}
```

---

### 2.2 InfoService

Liefert öffentliche Konfiguration für Clients.

```protobuf
service InfoService {
  rpc GetInfo(GetInfoRequest) returns (GetInfoResponse);
}
```

#### GetInfo

**Request:** `GetInfoRequest` (leer)

**Response:**

```protobuf
message GetInfoResponse {
  string version = 1;
  string environment = 2;
  AuthConfig auth = 3;
  UrlConfig urls = 4;
  FeatureFlags features = 5;
}

message AuthConfig {
  string issuer = 1;
  string client_id = 2;
}

message UrlConfig {
  string console = 1;
  string platform = 2;
  string docs = 3;
  string api = 4;
}

message FeatureFlags {
  bool registration = 1;
  bool social_login = 2;
}
```

---

### 2.3 UserService

User-Management mit CRUD-Operationen.

```protobuf
service UserService {
  rpc List(ListRequest) returns (ListResponse);
  rpc Get(GetRequest) returns (GetResponse);
  rpc GetCurrent(GetCurrentRequest) returns (GetCurrentResponse);
  rpc Create(CreateRequest) returns (CreateResponse);
  rpc Update(UpdateRequest) returns (UpdateResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
}
```

#### List

Paginierte Liste aller Benutzer.

**Request:**

```protobuf
message ListRequest {
  int32 page_size = 1;    // Max Items pro Seite
  string page_token = 2;  // Cursor für nächste Seite
}
```

**Response:**

```protobuf
message ListResponse {
  repeated User users = 1;
  string next_page_token = 2;
  int32 total_count = 3;
}
```

#### Get / GetCurrent

Einzelnen Benutzer abrufen (per ID oder aktueller Auth-Context).

**Response:**

```protobuf
message User {
  string id = 1;
  string email = 2;
  string name = 3;
  string role = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
}
```

#### Create / Update / Delete

Standard CRUD-Operationen für Benutzer.

---

### 2.4 StorageService

S3-kompatible Storage-Operationen.

```protobuf
service StorageService {
  rpc Upload(UploadRequest) returns (UploadResponse);
  rpc List(StorageServiceListRequest) returns (StorageServiceListResponse);
  rpc Delete(StorageServiceDeleteRequest) returns (StorageServiceDeleteResponse);
  rpc Head(StorageServiceHeadRequest) returns (StorageServiceHeadResponse);
  rpc GetPresignedUploadUrl(GetPresignedUploadUrlRequest) returns (GetPresignedUploadUrlResponse);
  rpc GetPresignedDownloadUrl(GetPresignedDownloadUrlRequest) returns (GetPresignedDownloadUrlResponse);
  rpc ListBuckets(ListBucketsRequest) returns (ListBucketsResponse);
  rpc CreateBucket(CreateBucketRequest) returns (CreateBucketResponse);
  rpc DeleteBucket(DeleteBucketRequest) returns (DeleteBucketResponse);
}
```

#### Upload

Datei hochladen.

**Request:**

```protobuf
message UploadRequest {
  bytes file = 1;           // Dateiinhalt
  string filename = 2;      // Dateiname
  string content_type = 3;  // MIME-Type
  optional string bucket = 4;
}
```

**Response:**

```protobuf
message UploadResponse {
  string key = 1;
  string bucket = 2;
  string url = 3;
  string etag = 4;
}
```

#### Presigned URLs

Für direkte Uploads/Downloads ohne Server-Proxy.

**Request:**

```protobuf
message GetPresignedUploadUrlRequest {
  string key = 1;
  optional string bucket = 2;
  int64 expires_in = 3;  // Sekunden
  optional string content_type = 4;
}
```

**Response:**

```protobuf
message GetPresignedUploadUrlResponse {
  string url = 1;
  int64 expires_in_secs = 2;
  string method = 3;
}
```

---

### 2.5 PeerService

ERY Peer-Management für P2P-Netzwerk und Multichain-Operationen.

```protobuf
service PeerService {
  rpc GetStatus(PeerServiceGetStatusRequest) returns (PeerServiceGetStatusResponse);
  rpc GetInfo(PeerServiceGetInfoRequest) returns (PeerServiceGetInfoResponse);
  rpc ListDerivedKeys(ListDerivedKeysRequest) returns (ListDerivedKeysResponse);
  rpc DeriveKey(DeriveKeyRequest) returns (DeriveKeyResponse);
  rpc EvaluateGateway(EvaluateGatewayRequest) returns (EvaluateGatewayResponse);
  rpc StartPeer(StartPeerRequest) returns (StartPeerResponse);
  rpc StopPeer(StopPeerRequest) returns (StopPeerResponse);
}
```

#### GetStatus

Aktueller Peer-Status mit Verbindungen und aktiven Sagas.

**Response:**

```protobuf
message PeerServiceGetStatusResponse {
  string peer_id = 1;
  DID did = 2;
  PeerState state = 3;
  repeated ConnectedChain connected_chains = 4;
  repeated DerivedWallet wallets = 5;
  GatewayStatus gateway_status = 6;
  repeated ActiveSagaSummary active_sagas = 7;
  google.protobuf.Timestamp started_at = 8;
  google.protobuf.Timestamp last_activity = 9;
}

enum PeerState {
  PEER_STATE_UNSPECIFIED = 0;
  PEER_STATE_STARTING = 1;
  PEER_STATE_RUNNING = 2;
  PEER_STATE_STOPPING = 3;
  PEER_STATE_STOPPED = 4;
  PEER_STATE_ERROR = 5;
}
```

#### EvaluateGateway

Gateway-Prädikate für Realm-Crossing evaluieren (Axiom PR3, PR6).

**Request:**

```protobuf
message EvaluateGatewayRequest {
  DID user = 1;
  RealmId source_realm = 2;
  RealmId target_realm = 3;
  bool verbose = 4;
}
```

**Response:**

```protobuf
message EvaluateGatewayResponse {
  bool allowed = 1;
  repeated PredicateResult predicates = 2;
  TrustVector6D original_trust = 3;
  TrustVector6D transformed_trust = 4;
  TrustDampeningMatrix applied_matrix = 5;
  string denial_reason = 6;
}
```

---

### 2.6 IntentService

Intent-Auflösung für Cross-Chain-Operationen (Axiom PR1).

```protobuf
service IntentService {
  rpc SubmitIntent(SubmitIntentRequest) returns (SubmitIntentResponse);
  rpc ResolveIntent(ResolveIntentRequest) returns (ResolveIntentResponse);
  rpc SimulateIntent(SimulateIntentRequest) returns (SimulateIntentResponse);
  rpc GetIntentStatus(GetIntentStatusRequest) returns (GetIntentStatusResponse);
  rpc ListIntents(ListIntentsRequest) returns (ListIntentsResponse);
  rpc CancelIntent(CancelIntentRequest) returns (CancelIntentResponse);
}
```

#### SubmitIntent

Natural-Language Intent einreichen.

**Request:**

```protobuf
message SubmitIntentRequest {
  string goal = 1;               // z.B. "Kaufe 50 kWh Strom"
  Budget budget = 2;
  optional ChainType source_chain = 3;
  optional RealmId target_realm = 4;
  optional double slippage_percent = 5;
  optional int32 timeout_seconds = 6;
  map<string, string> metadata = 7;
}

message Budget {
  string amount = 1;   // z.B. "100"
  string asset = 2;    // z.B. "USDC"
  ChainType chain = 3;
}
```

**Response:**

```protobuf
message SubmitIntentResponse {
  string intent_id = 1;
  IntentState state = 2;
  SagaPlan estimated_plan = 3;
  CostEstimate estimated_cost = 4;
  repeated string required_approvals = 5;
}

enum IntentState {
  INTENT_STATE_UNSPECIFIED = 0;
  INTENT_STATE_PENDING = 1;
  INTENT_STATE_RESOLVING = 2;
  INTENT_STATE_RESOLVED = 3;
  INTENT_STATE_EXECUTING = 4;
  INTENT_STATE_COMPLETED = 5;
  INTENT_STATE_FAILED = 6;
  INTENT_STATE_CANCELLED = 7;
}
```

---

### 2.7 SagaService

Saga-Management für atomare Cross-Chain-Transaktionen (Axiom PR2).

```protobuf
service SagaService {
  rpc ListSagas(ListSagasRequest) returns (ListSagasResponse);
  rpc GetSagaStatus(GetSagaStatusRequest) returns (GetSagaStatusResponse);
  rpc ExecuteSaga(ExecuteSagaRequest) returns (ExecuteSagaResponse);
  rpc CancelSaga(CancelSagaRequest) returns (CancelSagaResponse);
  rpc RollbackSaga(RollbackSagaRequest) returns (RollbackSagaResponse);
  rpc GetSagaHistory(GetSagaHistoryRequest) returns (GetSagaHistoryResponse);
  rpc StreamSagaUpdates(StreamSagaUpdatesRequest) returns (stream StreamSagaUpdatesResponse);
}
```

#### GetSagaStatus

Detaillierter Saga-Status mit Steps und Proofs.

**Response:**

```protobuf
message GetSagaStatusResponse {
  string saga_id = 1;
  string intent_id = 2;
  SagaState state = 3;
  repeated SagaStep steps = 4;
  HtlcStatus htlc_status = 5;
  bool rollback_available = 6;
  repeated string compensatable_steps = 7;
  google.protobuf.Timestamp created_at = 8;
  google.protobuf.Timestamp updated_at = 9;
  optional string error_message = 10;
}

enum SagaState {
  SAGA_STATE_UNSPECIFIED = 0;
  SAGA_STATE_PENDING = 1;
  SAGA_STATE_EXECUTING = 2;
  SAGA_STATE_WAITING_CONFIRMATION = 3;
  SAGA_STATE_ROLLING_BACK = 4;
  SAGA_STATE_COMPLETED = 5;
  SAGA_STATE_ROLLED_BACK = 6;
  SAGA_STATE_FAILED = 7;
  SAGA_STATE_CANCELLED = 8;
}
```

#### StreamSagaUpdates (Server-Streaming)

Echtzeit-Updates während Saga-Ausführung.

**Response (Stream):**

```protobuf
message StreamSagaUpdatesResponse {
  string saga_id = 1;
  SagaState state = 2;
  optional SagaStepUpdate step_update = 3;
  google.protobuf.Timestamp timestamp = 4;
}
```

---

### 2.8 EnvironmentService

Realm/Environment-Management (3-Schichten-Architektur).

```protobuf
service EnvironmentService {
  rpc ListEnvironments(ListEnvironmentsRequest) returns (ListEnvironmentsResponse);
  rpc GetEnvironmentTree(GetEnvironmentTreeRequest) returns (GetEnvironmentTreeResponse);
  rpc CreateEnvironment(CreateEnvironmentRequest) returns (CreateEnvironmentResponse);
  rpc JoinEnvironment(JoinEnvironmentRequest) returns (JoinEnvironmentResponse);
  rpc LeaveEnvironment(LeaveEnvironmentRequest) returns (LeaveEnvironmentResponse);
  rpc GetEnvironmentInfo(GetEnvironmentInfoRequest) returns (GetEnvironmentInfoResponse);
  rpc SwitchEnvironment(SwitchEnvironmentRequest) returns (SwitchEnvironmentResponse);
  rpc GetBootstrapStatus(GetBootstrapStatusRequest) returns (GetBootstrapStatusResponse);
}
```

#### CreateEnvironment

Neues Virtual Environment erstellen.

**Request:**

```protobuf
message CreateEnvironmentRequest {
  string name = 1;
  RealmId parent = 2;
  GovernanceType governance = 3;
  optional string axioms_ecl = 4;  // ECL Content
  optional CbdcConfig cbdc = 5;
  optional string description = 6;
}

enum GovernanceType {
  GOVERNANCE_TYPE_UNSPECIFIED = 0;
  GOVERNANCE_TYPE_DAO = 1;
  GOVERNANCE_TYPE_MULTI_SIG = 2;
  GOVERNANCE_TYPE_SINGLE = 3;
}
```

---

## III. Gemeinsame Typen

### 3.1 DID (Decentralized Identifier)

```protobuf
message DID {
  string namespace = 1;  // self, guild, spirit, thing
  string unique_id = 2;
  google.protobuf.Timestamp created_at = 3;
}
```

### 3.2 TrustVector6D

6-dimensionaler Trust-Vektor (Weltformel 𝕎).

```protobuf
message TrustVector6D {
  double reliability = 1;   // R: Zuverlässigkeit
  double integrity = 2;     // I: Integrität
  double competence = 3;    // C: Kompetenz
  double prestige = 4;      // P: Prestige
  double vigilance = 5;     // V: Vigilanz
  double omega = 6;         // Ω: Langzeit-Faktor
}
```

### 3.3 RealmId

```protobuf
message RealmId {
  string id = 1;  // z.B. "realm:erynoa:eu:energy"
}
```

### 3.4 ChainType

Unterstützte Blockchain-Typen.

```protobuf
enum ChainType {
  CHAIN_TYPE_UNSPECIFIED = 0;
  CHAIN_TYPE_ERYNOA = 1;
  CHAIN_TYPE_ETHEREUM = 2;
  CHAIN_TYPE_POLYGON = 3;
  CHAIN_TYPE_IOTA = 4;
  CHAIN_TYPE_SHIMMER = 5;
  CHAIN_TYPE_ARBITRUM = 6;
  CHAIN_TYPE_OPTIMISM = 7;
}
```

### 3.5 Algorithm

Kryptographische Algorithmen.

```protobuf
enum Algorithm {
  ALGORITHM_UNSPECIFIED = 0;
  ALGORITHM_ED25519 = 1;
  ALGORITHM_SECP256K1 = 2;
  ALGORITHM_BLS12_381 = 3;
}
```

---

## IV. Authentifizierung

### 4.1 WebAuthn/Passkey (REST)

Für Browser-basierte Authentifizierung:

| Endpoint                        | Methode | Beschreibung         |
| ------------------------------- | ------- | -------------------- |
| `/api/v1/auth/challenge`        | GET     | Challenge generieren |
| `/api/v1/auth/passkey/register` | POST    | Passkey registrieren |
| `/api/v1/auth/passkey/verify`   | POST    | Passkey verifizieren |

### 4.2 Token-basiert (Connect-RPC)

Authentifizierte Requests über `Authorization` Header:

```
Authorization: Bearer <token>
```

---

## V. Error Handling

### 5.1 Connect-RPC Error Codes

| Code | Name              | HTTP | Beschreibung             |
| ---- | ----------------- | ---- | ------------------------ |
| 0    | OK                | 200  | Erfolg                   |
| 1    | CANCELLED         | 499  | Request abgebrochen      |
| 2    | UNKNOWN           | 500  | Unbekannter Fehler       |
| 3    | INVALID_ARGUMENT  | 400  | Ungültiges Argument      |
| 5    | NOT_FOUND         | 404  | Ressource nicht gefunden |
| 7    | PERMISSION_DENIED | 403  | Keine Berechtigung       |
| 13   | INTERNAL          | 500  | Interner Server-Fehler   |
| 14   | UNAVAILABLE       | 503  | Service nicht verfügbar  |
| 16   | UNAUTHENTICATED   | 401  | Nicht authentifiziert    |

### 5.2 Error Response Format

```json
{
  "code": "not_found",
  "message": "User not found",
  "details": []
}
```

---

## VI. TypeScript Client

### 6.1 Installation

```bash
pnpm add @connectrpc/connect @connectrpc/connect-web @bufbuild/protobuf
```

### 6.2 Client Setup

```typescript
import { createConnectTransport } from "@connectrpc/connect-web";
import { createPromiseClient } from "@connectrpc/connect";
import { HealthService } from "./gen/erynoa/v1/health_connect";

const transport = createConnectTransport({
  baseUrl: "http://localhost:3000/api/v1/connect",
});

const client = createPromiseClient(HealthService, transport);

// Nutzung
const response = await client.check({});
console.log(response.status);
```

### 6.3 Mit Authentifizierung

```typescript
const authTransport = createConnectTransport({
  baseUrl: "http://localhost:3000/api/v1/connect",
  interceptors: [
    (next) => async (req) => {
      req.header.set("Authorization", `Bearer ${getToken()}`);
      return next(req);
    },
  ],
});
```

---

## VII. Axiom-Mapping

| Service            | Axiome  | Beschreibung          |
| ------------------ | ------- | --------------------- |
| PeerService        | PR1-PR6 | Peer-Axiome           |
| IntentService      | PR1     | Intent-Auflösung      |
| SagaService        | PR2     | Saga-Atomarität       |
| EnvironmentService | Κ1, Κ4  | Realm-Hierarchie      |
| UserService        | Κ10     | DID-Auth              |
| HealthService      | -       | Infrastruktur         |
| InfoService        | -       | Konfiguration         |
| StorageService     | Κ19     | Dezentrale Persistenz |

---

## VIII. Proto-Dateien

Die Proto-Definitionen befinden sich in:

```
backend/proto/erynoa/v1/
├── health.proto      # HealthService
├── info.proto        # InfoService
├── user.proto        # UserService
├── storage.proto     # StorageService
└── peer.proto        # PeerService, IntentService, SagaService, EnvironmentService
```

### Code-Generierung

```bash
# TypeScript generieren
pnpm buf generate

# Rust generieren (via build.rs)
cargo build
```

---

## IX. Versionierung

- **API Version:** `/api/v1/`
- **Proto Package:** `erynoa.v1`
- **Breaking Changes:** Nur in neuer Major-Version (`v2`)

---

_Erstellt: Februar 2026 | Basis: Proto-Definitionen & IPS v1.2.0_
