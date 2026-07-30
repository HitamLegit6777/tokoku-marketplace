// Demo data seeding. Populates a realistic UMKM store so first-run users see a
// fully working shop they can then customize. Idempotent: only runs when empty.
use crate::db::{self, Db};
use crate::models::{Banner, Product, Settings};
use crate::services::helpers::make_slug;
use anyhow::Result;
use serde_json::json;

pub fn seed_if_empty(db: &Db) -> Result<()> {
    if !db::is_empty_store(db)? {
        return Ok(());
    }
    tracing::info!("Seeding demo store data...");

    // --- Settings / demo payment config ---
    let mut s: Settings = db::get_settings(db)?;
    s.store_name = "Dapur Nusantara".into();
    s.tagline = "Camilan & Kopi Nusantara, dibuat dengan cinta".into();
    s.description =
        "UMKM makanan ringan dan kopi khas Indonesia. Bahan pilihan, produksi rumahan, rasa juara."
            .into();
    s.theme = "coffee".into();
    s.whatsapp = "081234567890".into();
    s.phone = "081234567890".into();
    s.email = "halo@dapurnusantara.id".into();
    s.address = "Jl. Melati No. 21, Yogyakarta".into();
    s.instagram = "dapurnusantara".into();
    s.currency = "Rp".into();
    s.shipping_flat = 15000;
    s.shipping_free_min = 150000;
    s.announcement =
        "Gratis ongkir untuk pembelian di atas Rp 150.000! Berlaku se-Indonesia.".into();
    s.announcement_on = true;
    s.payment_config = json!({
        "bank_transfer_enabled": true,
        "bank_name": "BCA",
        "bank_account_number": "1234567890",
        "bank_account_holder": "Dapur Nusantara",
        "qris_enabled": true,
        "qris_image_url": "",
        "cod_enabled": true,
        "ewallet_enabled": true,
        "ewallet_name": "GoPay / OVO / Dana",
        "ewallet_number": "081234567890",
        "midtrans_enabled": false
    })
    .to_string();
    db::update_settings(db, &s)?;

    // --- Categories ---
    let cat_snack = db::create_category(
        db,
        "Camilan",
        "camilan",
        "Keripik, kue kering, dan camilan renyah",
        "cookie",
        "",
    )?;
    let cat_coffee = db::create_category(
        db,
        "Kopi",
        "kopi",
        "Biji dan bubuk kopi Nusantara pilihan",
        "coffee",
        "",
    )?;
    let cat_drink = db::create_category(
        db,
        "Minuman",
        "minuman",
        "Minuman tradisional siap seduh",
        "cup",
        "",
    )?;
    let cat_gift = db::create_category(
        db,
        "Paket Hadiah",
        "paket-hadiah",
        "Hampers & paket spesial",
        "gift",
        "",
    )?;

    // --- Products ---
    let products = vec![
        ("Keripik Singkong Balado", cat_snack, 18000, 25000, 120, true,
         "Keripik singkong renyah dengan bumbu balado pedas manis khas rumahan.",
         "Renyah, pedas manis, bikin nagih", "pedas,keripik,camilan"),
        ("Kopi Arabika Gayo 200g", cat_coffee, 65000, 80000, 45, true,
         "Biji kopi Arabika Gayo single origin, roasting medium, aroma floral dengan body seimbang.",
         "Single origin Aceh, medium roast", "kopi,arabika,gayo,premium"),
        ("Kue Nastar Premium", cat_snack, 85000, 0, 60, true,
         "Nastar lumer dengan selai nanas asli, mentega premium, cocok untuk lebaran dan hadiah.",
         "Lumer, selai nanas asli", "kue,nastar,lebaran"),
        ("Kopi Robusta Lampung 250g", cat_coffee, 45000, 0, 70, false,
         "Robusta Lampung bold dan strong, cocok untuk espresso dan kopi tubruk.",
         "Bold & strong, cocok tubruk", "kopi,robusta,lampung"),
        ("Wedang Jahe Instan", cat_drink, 22000, 28000, 90, true,
         "Wedang jahe merah instan, hangat menyehatkan, tanpa pengawet.",
         "Hangat, menyehatkan, praktis", "jahe,wedang,herbal"),
        ("Keripik Tempe Original", cat_snack, 15000, 0, 150, false,
         "Keripik tempe tipis dan gurih, camilan sehat tinggi protein.",
         "Tipis, gurih, tinggi protein", "keripik,tempe,sehat"),
        ("Bandrek Susu Instan", cat_drink, 25000, 0, 80, false,
         "Bandrek rempah dengan susu, minuman hangat khas Sunda.",
         "Rempah + susu, khas Sunda", "bandrek,rempah,hangat"),
        ("Hampers Lebaran Spesial", cat_gift, 250000, 300000, 25, true,
         "Paket hampers berisi nastar, kastengel, kopi Gayo, dan wedang jahe dalam kemasan eksklusif.",
         "Isi 4 produk pilihan, kemasan mewah", "hampers,lebaran,hadiah,paket"),
        ("Kastengel Keju Premium", cat_snack, 95000, 0, 40, false,
         "Kastengel keju edam dengan tekstur renyah dan rasa gurih yang kaya.",
         "Keju edam, renyah gurih", "kue,kastengel,keju"),
        ("Kopi Susu Gula Aren Botol", cat_drink, 28000, 35000, 100, true,
         "Kopi susu gula aren siap minum, manis pas dengan aroma kopi yang kuat.",
         "Siap minum, gula aren asli", "kopi,susu,gula-aren"),
        ("Manisan Mangga Kering", cat_snack, 20000, 0, 110, false,
         "Manisan mangga kering manis asam, camilan buah alami tanpa pemanis buatan.",
         "Manis asam, buah alami", "manisan,mangga,buah"),
        ("Teh Rosella Kering", cat_drink, 30000, 0, 65, false,
         "Kelopak bunga rosella kering untuk teh herbal kaya antioksidan.",
         "Herbal, kaya antioksidan", "teh,rosella,herbal"),
    ];

    for (name, cat, price, compare, stock, featured, desc, short, tags) in products {
        let p = Product {
            name: name.into(),
            slug: make_slug(name),
            price,
            compare_price: compare,
            stock,
            track_stock: true,
            category_id: Some(cat),
            description: desc.into(),
            short_desc: short.into(),
            tags: tags.into(),
            is_active: true,
            is_featured: featured,
            images: "[]".into(),
            ..Default::default()
        };
        let id = db::upsert_product(db, &p, None)?;
        // add a couple of sample reviews for social proof
        let _ = db::create_review(db, id, "Rina", 5, "Enak banget, pasti pesan lagi!");
        let _ = db::create_review(db, id, "Budi", 4, "Kualitas oke, pengiriman cepat.");
    }

    // --- Banners ---
    let banners = vec![
        Banner {
            title: "Kopi Nusantara Pilihan".into(),
            subtitle: "Diseduh dari biji terbaik Aceh sampai Lampung".into(),
            button_text: "Belanja Kopi".into(),
            link_url: "/category/kopi".into(),
            sort_order: 1,
            is_active: true,
            ..Default::default()
        },
        Banner {
            title: "Gratis Ongkir se-Indonesia".into(),
            subtitle: "Minimal belanja Rp 150.000, berlaku semua produk".into(),
            button_text: "Mulai Belanja".into(),
            link_url: "/products".into(),
            sort_order: 2,
            is_active: true,
            ..Default::default()
        },
    ];
    for b in &banners {
        db::create_banner(db, b)?;
    }

    // --- Pages ---
    db::upsert_page(db, "Tentang Kami", "tentang-kami",
        "Dapur Nusantara adalah UMKM keluarga yang berdiri sejak 2019.\n\nKami berkomitmen menghadirkan camilan dan kopi khas Indonesia dengan bahan pilihan dan proses produksi yang higienis. Setiap produk dibuat dengan penuh perhatian agar sampai ke tangan Anda dalam kondisi terbaik.", None)?;
    db::upsert_page(db, "Cara Pemesanan", "cara-pemesanan",
        "1. Pilih produk dan masukkan ke keranjang.\n2. Klik checkout dan isi data pengiriman.\n3. Pilih metode pembayaran.\n4. Lakukan pembayaran dan upload bukti transfer.\n5. Pesanan diproses dan dikirim.", None)?;
    db::upsert_page(db, "Kebijakan Pengiriman", "pengiriman",
        "Pesanan diproses 1x24 jam pada hari kerja. Pengiriman menggunakan ekspedisi rekanan (JNE, J&T, SiCepat). Estimasi tiba 2-4 hari kerja tergantung lokasi.", None)?;

    tracing::info!("Demo store seeded successfully.");
    Ok(())
}
