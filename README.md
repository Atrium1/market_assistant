# Market Asistanı

Küçük marketler için Windows masaüstü uygulaması: barkod ile fiyat bakma, sepet ve satış kaydı.

Çalışanların kod çalıştırmasına gerek yoktur; geliştirme bilgisayarında bir kez kurulur, sonra `.msi` / NSIS kurulum dosyası ile dağıtılır.

## Özellikler (V1)

- **Kasa:** Satış / Fiyat sor / İade (placeholder) — giriş gerekmez
- **Ürünler:** çalışanlar listeyi görür; ekle/sil/fiyat için **Düzenleme modu** + sahip girişi
- **Ayarlar:** mağaza adı ve dükkan kullanıcı adı/şifresi (sahip oturumu gerekir)
- Yerel **SQLite** + migration’lar
- API sözleşmesi → [`docs/api-contract.md`](docs/api-contract.md)

## Sahip girişi ve fabrika kurtarma

- Kasa her zaman açıktır (çalışan modu).
- Ürün değiştirme / Ayarlar için kullanıcı adı + şifre gerekir.
- Her kurulumda gömülü **fabrika (kurtarma)** hesabı vardır; dükkan şifresi unutulursa bununla giriş yapıp Ayarlar’dan yeni hesap belirlenir.
- Dağıtımdan önce [`apps/desktop/src-tauri/src/auth/factory.rs`](apps/desktop/src-tauri/src/auth/factory.rs) içindeki `CHANGE_ME_USER` / `CHANGE_ME_PASS` değerlerini değiştirin. Gerçek sırları herkese açık repoya koymayın.
- Fabrika hesabı tüm kopyalarda aynıdır (bilinçli kurtarma arka kapısı); sadece sizin bilmeniz gerekir.

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
