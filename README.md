# Market Asistanı

Küçük marketler için Windows masaüstü uygulaması: barkod ile fiyat bakma, sepet ve satış kaydı.

Çalışanların kod çalıştırmasına gerek yoktur; geliştirme bilgisayarında bir kez kurulur, sonra `.msi` / NSIS kurulum dosyası ile dağıtılır.

## Özellikler (V1 iskelet)

- **Ürünler:** barkod, ad, fiyat (kuruş olarak saklanır)
- **Kasa:** Satış / Fiyat sor / İade (placeholder)
- Sepet: satır sil, adet, sonunu sil, sepeti iptal, satışı tamamla
- Yerel **SQLite** + migration’lar
- Tauri komutları ileride mobil/HTTP API ile aynı sözleşmeye map edilir → [`docs/api-contract.md`](docs/api-contract.md)

## Geliştirme (geliştirici)

### Gereksinimler

- Node.js 20+
- Rust (rustup) — [https://rustup.rs](https://rustup.rs)
- Windows’ta kurulum için: [Tauri Windows önkoşulları](https://v2.tauri.app/start/prerequisites/) (WebView2 genelde yüklü)

macOS’ta geliştirme yapılabilir; **ilk dağıtım hedefi yalnızca Windows**.

### Çalıştırma

```bash
cd apps/desktop
npm install
npm run tauri:dev
```

Uygulama açılır; veri dosyası işletim sisteminin app data klasörüne yazılır  
(ör. Windows: `%APPDATA%\com.marketassistant.app\market_assistant.db`).

### Frontend-only (UI denemesi)

Tauri komutları tarayıcıda çalışmaz. Sadece Vite için:

```bash
cd apps/desktop
npm run dev
```

### Windows kurulum dosyası üretme

Windows makinede:

```bash
cd apps/desktop
npm install
npm run tauri:build
```

Çıktılar yaklaşık olarak:

- `apps/desktop/src-tauri/target/release/bundle/nsis/`
- `apps/desktop/src-tauri/target/release/bundle/msi/`

Bu dosyayı arkadaşlarınıza gönderirsiniz; kurulumdan sonra masaüstü kısayolu ile açılır.

## Proje yapısı

```text
apps/desktop/          Tauri 2 + React (Türkçe UI)
  src/                 Ön yüz
  src-tauri/           Rust domain, SQLite, komutlar
packages/              İleride paylaşılan istemci tipleri
docs/api-contract.md   Komut ↔ REST sözleşmesi
```

## Sonraki adımlar (plan)

- Stok düşümü (`inventory` tablosu hazır)
- Günlük satış / kasa raporu
- Tam iade akışı
- Müşteri ikinci ekran
- Mobil istemci + isteğe bağlı bulut Postgres
