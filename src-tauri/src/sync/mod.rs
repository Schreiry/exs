// Sync primitives. The full P2P/WebSocket layer from Exsul was intentionally
// NOT ported into the new core (it belongs to a later phase). Only the Hybrid
// Logical Clock — needed for the event ledger — is kept here.
pub mod hlc;
