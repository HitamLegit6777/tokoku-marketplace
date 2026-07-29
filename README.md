# TokoKu 🛍️

**Platform toko online siap pakai untuk UMKM Indonesia.** Dibuat dengan Rust (Axum + SQLite), ringan, cepat, dan bisa dijalankan dari satu file binary. Cocok untuk berjualan online tanpa perlu tim developer.

Panel admin no-code untuk mengelola produk, pesanan, pembayaran, tampilan toko, sampai **Laporan Keuangan** lengkap.

---

## ✨ Fitur

### Toko (storefront)
- Halaman depan dengan banner/hero, kategori, produk unggulan, testimoni
- Katalog produk dengan pencarian, filter kategori, dan sorting
- Halaman detail produk, keranjang, dan checkout
- Konfirmasi pesanan + halaman lacak pesanan (tracking)
- Halaman statis (Tentang, Kebijakan, dll) yang bisa diatur dari admin
- 11 tema tampilan siap pakai, ganti tema tanpa ngoding

### SEO (siap mesin pencari)
- Tag `title`, `description`, `keywords`, dan `canonical` per halaman
- Open Graph + Twitter Card (judul, deskripsi, gambar) untuk preview saat dibagikan
- Structured data JSON-LD: `Store` di semua halaman, `Product` + `Offer` (harga IDR, stok) di halaman produk
- `robots.txt` otomatis (blokir area admin/checkout) + `sitemap.xml` dinamis berisi produk, kategori, dan halaman
- URL absolut mengikuti `BASE_URL` yang diatur di `.env`

### Admin (`/admin`)
- **Dashboard** — ringkasan pendapatan, pesanan, produk, pelanggan + grafik 7 hari
- **Laporan Keuangan** (`/admin/finance`) — pendapatan kotor, HPP/modal, laba kotor & bersih, margin, rata-rata nilai order, breakdown per metode pembayaran, grafik pendapatan 12 bulan, produk penyumbang pendapatan teratas, plus info tagihan belum dibayar & refund. Filter periode 7 / 30 / 90 hari & 1 tahun.
- **Produk & Kategori** — CRUD lengkap, upload gambar, stok, harga modal
- **Banner, Kupon, Halaman** — kelola konten toko
- **Pesanan** — update status pesanan & pembayaran, nomor resi
- **Pembayaran** — Transfer Bank, QRIS, COD, E-Wallet, dan Midtrans
- **Tampilan & Tema**, **Pengaturan toko** (nama, kontak, ongkir, pajak, dll)

---

## 🧰 Teknologi

| Bagian        | Teknologi                              |
|---------------|----------------------------------------|
| Bahasa        | Rust (edition 2021)                    |
| Web framework | [Axum](https://github.com/tokio-rs/axum) 0.7 |
| Template      | [Askama](https://github.com/djc/askama) (server-side HTML) |
| Database      | SQLite via `rusqlite` (bundled) + `r2d2` pool |
| Auth          | Sesi (`tower-sessions`) + hashing `argon2` |
| Lainnya       | `tokio`, `tower-http`, `serde`, `chrono` |

Tanpa build step frontend — HTML/CSS/JS statis, langsung jalan.

---

## 🚀 Menjalankan

### Prasyarat
- [Rust toolchain](https://rustup.rs/) (stable)

### Langkah
```bash
# 1. Clone
git clone https://github.com/HitamLegit6777/tokoku-marketplace.git
cd tokoku-marketplace

# 2. (Opsional) siapkan konfigurasi
cp .env.example .env
# ubah nilai di .env sesuai kebutuhan

# 3. Jalankan
cargo run --release
```

Server berjalan di **http://localhost:8080** (bisa diubah lewat `PORT`).

Saat pertama kali dijalankan, database akan otomatis dibuat dan diisi data contoh, serta akun admin default dibuat.

### Akun admin default
| Field    | Nilai      |
|----------|------------|
| URL      | `/admin`   |
| Username | `admin`    |
| Password | `admin123` |

> ⚠️ **Segera ganti password** setelah login pertama. Atur juga lewat `ADMIN_USER` / `ADMIN_PASSWORD` di `.env` sebelum boot pertama.

---

## ⚙️ Konfigurasi (`.env`)

| Variabel         | Default                      | Keterangan                        |
|------------------|------------------------------|-----------------------------------|
| `PORT`           | `8080`                       | Port server                       |
| `BASE_URL`       | `http://localhost:8080`      | URL dasar untuk link              |
| `DATABASE_PATH`  | `data/tokoku.db`             | Lokasi file SQLite                |
| `ADMIN_USER`     | `admin`                      | Username admin (boot pertama)     |
| `ADMIN_PASSWORD` | `admin123`                   | Password admin (boot pertama)     |
| `RUST_LOG`       | `tokoku=info`                | Level log                         |

---

## 📁 Struktur Proyek

```
├── src/
│   ├── main.rs            # entrypoint + routing
│   ├── db.rs              # query & agregasi (termasuk finance_report)
│   ├── handlers/          # admin, store, cart, checkout, setup
│   ├── services/          # auth, cart, payment, themes, seed, upload
│   ├── models/            # struct data
│   ├── filters.rs         # filter template (rupiah, dsb)
│   └── icons.rs           # ikon SVG inline
├── templates/             # Askama HTML (admin/, store/, setup/, partials/)
├── static/                # css, js, gambar
├── migrations/            # skema SQL
└── Cargo.toml
```

---

## 💾 Data

Database SQLite disimpan di `data/tokoku.db` (tidak di-commit ke repo). Skema awal ada di `migrations/001_init.sql`. Data contoh otomatis di-seed pada boot pertama jika toko masih kosong.

---

## 📄 Lisensi

Dirilis di bawah lisensi [MIT](LICENSE). Bebas digunakan, dimodifikasi, dan didistribusikan.
