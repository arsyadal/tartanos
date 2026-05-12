# Tartanos

Tartanos adalah browser desktop ringan berbasis Rust. Project ini berfokus pada pengalaman browsing inti yang cepat, sederhana, dan familier dengan memanfaatkan `wry`/WebView sebagai rendering layer.

## Fitur saat ini

- Multi-tab dasar
- Address bar untuk URL dan pencarian
- Navigasi back, forward, reload
- Bookmark sederhana
- History browsing
- Halaman start, history, dan settings
- Shortcut keyboard umum
- Indikasi tab loading/audible
- Dukungan download dasar

## Tech stack

- Rust 2024
- [`wry`](https://crates.io/crates/wry) untuk WebView
- [`tao`](https://crates.io/crates/tao) untuk window/event loop
- `serde`/`serde_json` untuk state dan pesan UI

## Menjalankan project

Pastikan Rust sudah terpasang.

```bash
cargo run
```

Untuk cek kompilasi:

```bash
cargo check
```

Jika menggunakan alias dari `.cargo/config.toml`:

```bash
cargo dev
cargo dev-check
```

## Struktur project

```text
src/
  main.rs            Entry point aplikasi
  app.rs             Core browser app, tab, navigation, storage, command handling
  toolbar.rs         UI toolbar dan tab strip
  start_page.rs      Halaman new tab/start
  history_page.rs    Halaman history
  settings_page.rs   Halaman settings
prd.md               Product Requirements Document
```

## Catatan keamanan repository

Repository ini ditujukan untuk publik. File kredensial, secret, token, sertifikat private, file `.env`, dan data runtime lokal tidak boleh dipush. Lihat `.gitignore` untuk daftar file sensitif yang dikecualikan.

## License

Belum ditentukan.
