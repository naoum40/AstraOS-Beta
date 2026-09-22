# AstraPass

AstraOS password manager — built with **Rust + GTK4-rs + libadwaita**.

AstraPass is the Proton Pass–inspired credential vault for AstraOS ISO 4
(v0.4). It stores credentials locally in an encrypted vault file, sealed
with **AES-GCM 256-bit** authenticated encryption and unlocked with a
master password hashed via **PBKDF2-HMAC-SHA256** (150,000 iterations).

## Features

- **3-pane layout** (sidebar / entry list / detail editor), modeled after
  Proton Pass.
- **AES-GCM 256-bit** authenticated encryption of the vault payload.
- **PBKDF2-HMAC-SHA256** master password key derivation (150k rounds).
- **First-time setup** with live password strength indicator.
- **Unlock screen** with masked input + wrong-password feedback.
- **Search + filter** (All / Favorites / Trash) in the sidebar.
- **Per-entry fields**: title, username, password (with show/hide toggle),
  notes (multi-line), favorite flag.
- **Glassmorphism dark theme** with violet→magenta gradient accents.

## Security model

- The vault file lives at `~/.config/astra/astrapass/vault.bin`.
- The file is a JSON document wrapping three base64-encoded fields:
  - `salt`      — 16 random bytes (one-time, generated on vault creation).
  - `nonce`     — 12 random bytes (regenerated on every save).
  - `ciphertext`— AES-GCM ciphertext + 16-byte authentication tag of
                  the inner vault JSON (`{ "version": 1, "entries": [...] }`).
- The AES-256 key is derived from `PBKDF2-HMAC-SHA256(master_password, salt, 150_000)`.
- Wrong master password ⇒ GCM tag verification fails ⇒ unlock is
  rejected. No timing-leak of which byte mismatched.

## Build

```sh
cargo build --release
```

Dependencies (Debian/Arch package names):

- `gtk4` ≥ 4.14 (libgtk-4-dev / gtk4)
- `libadwaita-1` ≥ 1.5 (libadwaita-1-dev / libadwaita-1)

## Run

```sh
astra-pass
```

First launch will show the setup screen. Subsequent launches will show
the unlock screen.

## Vault file format

```json
{
  "salt":      "<base64>",
  "nonce":     "<base64>",
  "ciphertext":"<base64>"
}
```

Inner (decrypted) payload:

```json
{
  "version": 1,
  "entries": [
    {
      "id":         "32-hex-chars",
      "title":      "GitHub",
      "username":   "octocat",
      "password":   "••••••••",
      "notes":      "personal",
      "favorite":   true,
      "created_at": 1700000000
    }
  ]
}
```

## License

MIT — © AstraOS Project.
