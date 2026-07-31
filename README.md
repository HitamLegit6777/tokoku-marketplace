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
- **30 tema kontekstual** dengan 9 personality layout, ganti tema tanpa ngoding

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

## 🎨 Galeri 30 Tema Kontekstual

Setiap tema memiliki identitas UI sendiri, bukan hanya pergantian warna. Header, komposisi hero, motif, rasio gambar, kartu produk, grid, kategori, CTA, dan detail produk disesuaikan dengan konteks usaha.

<details>
<summary><strong>Kuliner & Kebutuhan Harian</strong> (6 tema)</summary>

<table>
<tr>
<td width="50%"><img src="docs/themes/sunset.webp" alt="Tema Sunset Glow"><br><strong>Sunset Glow</strong><br><sub>Hangat & ramah, cocok untuk kuliner dan fashion</sub></td>
<td width="50%"><img src="docs/themes/coffee.webp" alt="Tema Coffee House"><br><strong>Coffee House</strong><br><sub>Cozy & artisan, cocok untuk kopi, roti, dan produk handmade</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/grocery.webp" alt="Tema Fresh Grocery"><br><strong>Fresh Grocery</strong><br><sub>Segar dan praktis untuk sembako, sayur, dan supermarket</sub></td>
<td width="50%"><img src="docs/themes/bakery.webp" alt="Tema Sweet Bakery"><br><strong>Sweet Bakery</strong><br><sub>Lezat dan mengundang untuk bakery, dessert, dan kafe</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/seafood.webp" alt="Tema Ocean Catch"><br><strong>Ocean Catch</strong><br><sub>Segar dan berani untuk seafood, frozen food, dan restoran</sub></td>
<td width="50%"><img src="docs/themes/heritage.webp" alt="Tema Heritage Market"><br><strong>Heritage Market</strong><br><sub>Klasik bernuansa lokal untuk kerajinan, jamu, dan produk artisan</sub></td>
</tr>
</table>

</details>

<details>
<summary><strong>Fashion, Beauty & Lifestyle</strong> (7 tema)</summary>

<table>
<tr>
<td width="50%"><img src="docs/themes/berry.webp" alt="Tema Berry Bloom"><br><strong>Berry Bloom</strong><br><sub>Feminin & elegan, cocok untuk kecantikan dan aksesoris</sub></td>
<td width="50%"><img src="docs/themes/beauty.webp" alt="Tema Rose Beauty"><br><strong>Rose Beauty</strong><br><sub>Lembut dan premium untuk skincare, salon, dan kosmetik</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/modest.webp" alt="Tema Modest Grace"><br><strong>Modest Grace</strong><br><sub>Anggun dan bersih untuk hijab, busana muslim, dan modest wear</sub></td>
<td width="50%"><img src="docs/themes/jewelry.webp" alt="Tema Golden Jewel"><br><strong>Golden Jewel</strong><br><sub>Eksklusif untuk perhiasan, jam tangan, dan hadiah premium</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/nordic.webp" alt="Tema Nordic Atelier"><br><strong>Nordic Atelier</strong><br><sub>Editorial tenang dengan ruang lega untuk fashion dan interior</sub></td>
<td width="50%"><img src="docs/themes/thrift.webp" alt="Tema Retro Thrift"><br><strong>Retro Thrift</strong><br><sub>Nostalgik dan unik untuk thrift, vintage, dan preloved</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/florist.webp" alt="Tema Bloom Florist"><br><strong>Bloom Florist</strong><br><sub>Romantis dan organik untuk bunga, hampers, dan wedding gift</sub></td>
<td></td>
</tr>
</table>

</details>

<details>
<summary><strong>Teknologi, Otomotif & Profesional</strong> (6 tema)</summary>

<table>
<tr>
<td width="50%"><img src="docs/themes/ocean.webp" alt="Tema Ocean Breeze"><br><strong>Ocean Breeze</strong><br><sub>Bersih & profesional, cocok untuk elektronik dan jasa</sub></td>
<td width="50%"><img src="docs/themes/aurora.webp" alt="Tema Aurora Glass"><br><strong>Aurora Glass</strong><br><sub>Futuristik bercahaya untuk digital product, gadget, dan gaming</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/gaming.webp" alt="Tema Cyber Gaming"><br><strong>Cyber Gaming</strong><br><sub>Imersif dan neon untuk gaming gear, PC, dan komunitas esports</sub></td>
<td width="50%"><img src="docs/themes/automotive.webp" alt="Tema Torque Garage"><br><strong>Torque Garage</strong><br><sub>Tegas dan maskulin untuk otomotif, sparepart, dan bengkel</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/b2b.webp" alt="Tema Industrial Pro"><br><strong>Industrial Pro</strong><br><sub>Efisien dan terpercaya untuk supplier, manufaktur, dan B2B</sub></td>
<td width="50%"><img src="docs/themes/property.webp" alt="Tema Urban Estate"><br><strong>Urban Estate</strong><br><sub>Modern dan kredibel untuk properti, arsitektur, dan interior</sub></td>
</tr>
</table>

</details>

<details>
<summary><strong>Keluarga, Hobi & Kesehatan</strong> (7 tema)</summary>

<table>
<tr>
<td width="50%"><img src="docs/themes/forest.webp" alt="Tema Forest Fresh"><br><strong>Forest Fresh</strong><br><sub>Natural & organik, cocok untuk produk sehat dan herbal</sub></td>
<td width="50%"><img src="docs/themes/candy.webp" alt="Tema Candy Pop"><br><strong>Candy Pop</strong><br><sub>Ceria & playful, cocok untuk produk anak dan mainan</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/baby.webp" alt="Tema Little Cloud"><br><strong>Little Cloud</strong><br><sub>Manis dan aman untuk perlengkapan bayi dan ibu</sub></td>
<td width="50%"><img src="docs/themes/pets.webp" alt="Tema Happy Paws"><br><strong>Happy Paws</strong><br><sub>Ramah dan ceria untuk pet shop, grooming, dan pakan hewan</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/pharmacy.webp" alt="Tema MediCare Clean"><br><strong>MediCare Clean</strong><br><sub>Klinis dan terpercaya untuk apotek, kesehatan, dan alat medis</sub></td>
<td width="50%"><img src="docs/themes/sport.webp" alt="Tema Active Sport"><br><strong>Active Sport</strong><br><sub>Dinamis dan kuat untuk olahraga, gym, dan outdoor</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/books.webp" alt="Tema Literary Books"><br><strong>Literary Books</strong><br><sub>Hangat dan intelektual untuk buku, alat tulis, dan edukasi</sub></td>
<td></td>
</tr>
</table>

</details>

<details>
<summary><strong>Interior & Gaya Eksperimental</strong> (4 tema)</summary>

<table>
<tr>
<td width="50%"><img src="docs/themes/midnight.webp" alt="Tema Midnight Luxe"><br><strong>Midnight Luxe</strong><br><sub>Mewah & premium dengan nuansa gelap, cocok untuk brand eksklusif</sub></td>
<td width="50%"><img src="docs/themes/mono.webp" alt="Tema Mono Minimal"><br><strong>Mono Minimal</strong><br><sub>Minimalis hitam-putih, fokus pada produk (gaya butik modern)</sub></td>
</tr>
<tr>
<td width="50%"><img src="docs/themes/brutal.webp" alt="Tema Neo Brutal"><br><strong>Neo Brutal</strong><br><sub>Berani, kontras, dan anti-mainstream untuk streetwear dan kreator</sub></td>
<td width="50%"><img src="docs/themes/furniture.webp" alt="Tema Warm Living"><br><strong>Warm Living</strong><br><sub>Tenang dan estetik untuk furnitur, dekorasi, dan home living</sub></td>
</tr>
</table>

</details>

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

Saat pertama kali dijalankan, database akan otomatis dibuat dan diisi data contoh, serta akun admin dibuat.

### Akun admin pertama

URL admin adalah `/admin`. Atur `ADMIN_USER` dan `ADMIN_PASSWORD` yang kuat di `.env` **sebelum boot pertama**. Jika `ADMIN_PASSWORD` tidak diatur, aplikasi membuat password acak 24 karakter dan menampilkannya sekali di log startup. Segera ganti password melalui Pengaturan setelah login.

---

## ⚙️ Konfigurasi (`.env`)

| Variabel         | Default                      | Keterangan                        |
|------------------|------------------------------|-----------------------------------|
| `PORT`           | `8080`                       | Port server                       |
| `BASE_URL`       | `http://localhost:8080`      | URL dasar untuk link              |
| `DATABASE_PATH`  | `data/tokoku.db`             | Lokasi file SQLite                |
| `SESSION_DATABASE_PATH` | `data/sessions.db`     | Lokasi sesi persisten             |
| `ADMIN_USER`     | `admin`                      | Username admin (boot pertama)     |
| `ADMIN_PASSWORD` | acak jika kosong             | Password admin (boot pertama)     |
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
