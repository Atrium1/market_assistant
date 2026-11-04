# Market Asistanı

Küçük marketler için Windows masaüstü uygulaması: barkod ile fiyat bakma, sepet ve satış kaydı.

Çalışanların kod çalıştırmasına gerek yoktur; geliştirme bilgisayarında bir kez kurulur, sonra `.msi` / NSIS kurulum dosyası ile dağıtılır.

## Özellikler (V1)

- **Kasa:** Satış / Fiyat sor / İade (placeholder) — giriş gerekmez. Ödeme **Nakit (F8)** veya **Kart (F9)**; para üstü hesabı; üstte "Bugün: ₺X · N satış"
- **Raporlar:** gün sonu kasa raporu, satış geçmişi, satış iptali (sahip girişi gerekir)
- **Ürünler:** çalışanlar listeyi görür ve ad/barkod ile arar; ekle/sil/fiyat için **Düzenleme modu** + sahip girişi
- **Toplu işlemler:** Excel/CSV’den ürün içe aktarma (önizlemeli), Excel’e dışa aktarma, boş şablon
- **Ayarlar:** mağaza adı, dükkan kullanıcı adı/şifresi, **yedekleme / geri yükleme** (sahip oturumu gerekir)
- Yerel **SQLite** + migration’lar
- API sözleşmesi → [`docs/api-contract.md`](docs/api-contract.md)

## Sahip girişi ve fabrika kurtarma

- Kasa her zaman açıktır (çalışan modu).
- Ürün değiştirme / Ayarlar için kullanıcı adı + şifre gerekir.
- Her kurulumda gömülü **fabrika (kurtarma)** hesabı vardır; dükkan şifresi unutulursa bununla giriş yapıp Ayarlar’dan yeni hesap belirlenir.
- Dağıtımdan önce [`apps/desktop/src-tauri/src/auth/factory.rs`](apps/desktop/src-tauri/src/auth/factory.rs) içindeki `CHANGE_ME_USER` / `CHANGE_ME_PASS` değerlerini değiştirin. Gerçek sırları herkese açık repoya koymayın.
- Fabrika hesabı tüm kopyalarda aynıdır (bilinçli kurtarma arka kapısı); sadece sizin bilmeniz gerekir.

## Satış, gün sonu ve iptal

- **Fiyat her zaman sistemden alınır.** Sepette ürün dururken fiyatı değiştirilirse ödeme alınmaz; sepet yeni fiyatlarla güncellenir ve değişen ürünler gösterilir. Kasiyer toplamı müşteriye tekrar söyleyip yeniden Nakit/Kart'a basar. Silinmiş ürünler sepetten çıkarılır.
- **Fiş numarası:** her gün 1'den başlar (`04.11.2026 #0012`). Satış sonrası ekranda görünür; iptal veya soru için bu numarayı kullanın.
- **Raporlar → Gün sonu:** seçilen günün toplamı, nakit / kart ayrımı, iptal edilenler, ortalama sepet, saatlere göre satış, en çok satan ürünler ve son 7 gün. **Kasa sayımı** kutusuna çekmecedeki nakdi yazınca fark (fazla / eksik) gösterilir; bu bilgi kaydedilmez. **Yazdır** ile çıktı alınabilir.
- **Raporlar → Satış geçmişi:** tarih aralığı ve ödeme türüne göre liste; satıra tıklayınca ürünler görünür.
- **Satış iptali:** satış detayında **İptal et** → neden yazılır → onay. Satış silinmez; raporlarda "iptal edilen" olarak ayrıca gösterilir ve toplamdan düşülür. Bugünden eski bir satış iptal edilirse o günün raporu değişir; uygulama bunun için uyarır.
- Çalışanlar Kasa'da sadece bugünün toplamını görür; rapor, geçmiş ve iptal sahip girişi ister.
- **Bilgisayarın tarih ve saati doğru olmalı.** Satışın hangi güne yazılacağı satış anındaki bilgisayar saatine göre belirlenir; saat yanlışsa satış yanlış güne düşer.

## Yedekleme

Tüm ürünler ve satışlar bilgisayarda tek bir dosyadadır. Disk bozulursa veya bilgisayar çalınırsa bu dosya da gider.

- **Otomatik yedek:** uygulama her gün ilk açılışta `backups` klasörüne bir kopya alır (son 7 gün saklanır). Geri yükleme ve içe aktarma öncesinde de otomatik kopya alınır. Bunlar **aynı diskte** durur: yanlışlıkla silme / hatalı içe aktarmaya karşı korur, **disk arızasına karşı korumaz**.
- **Elle yedek (önerilir):** Ayarlar → Yedekleme → **Yedekle** ile dosyayı haftada bir **USB belleğe** veya başka bir bilgisayara kaydedin.
- **Geri yükleme:** Ayarlar → **Yedekten geri yükle**. Yedekteki ürün/satış sayısı gösterilir, onaydan sonra mevcut verinin yerine geçer (mevcut veri önce otomatik yedeklenir). Dükkan şifresi de yedekteki hâline döner; unutulduysa fabrika hesabı ile girilir.
- Yeni bilgisayara taşıma: eski bilgisayarda Yedekle → yeni bilgisayara uygulamayı kurun → Yedekten geri yükle.

## Excel / CSV ile ürün yükleme

Ürünler → Düzenleme modu → **Toplu işlemler**.

1. **Şablon indir** ile örnek Excel dosyasını alın (sütunlar: `Barkod`, `Ürün adı`, `Fiyat`). Örnek satırı silip kendi ürünlerinizi yazın.
2. **Excel/CSV içe aktar** ile dosyayı seçin. Önce bir önizleme gelir: yeni / güncellenecek / değişmeyen / hatalı satırlar. Hiçbir şey onaylamadan kaydedilmez.
3. **N ürünü uygula** ile kaydedin. Hatalı satırlar atlanır.

Dikkat edilecekler:

- **Barkod sütununu metin yapın.** Excel uzun sayıları `8,69E+12` gibi bilimsel gösterime çevirir veya baştaki `0`’ları siler. Şablonda bu sütun zaten metin biçimindedir; kendi dosyanızda sütunu seçip *Hücreleri Biçimlendir → Metin* yapın. Bilimsel gösterimli barkodlar hatalı sayılır.
- Fiyat `12,50`, `12.50`, `1.234,50` veya `₺12,50` olabilir.
- Aynı barkod dosyada iki kez geçerse iki satır da hatalı sayılır.
- Daha önce silinmiş bir ürünün barkodu dosyada varsa ürün geri getirilir.
- Başka programdan alınan CSV dosyaları (`;` veya `,` ayraçlı, Türkçe karakterli) desteklenir.
- **Dışa aktar** mevcut listeyi aynı biçimde Excel’e yazar; düzenleyip tekrar içe aktarabilirsiniz.

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
- Tam iade akışı (rapordaki `returns_total` alanı hazır)
- Gün sonu kapanışını / kasa sayımını kaydetme
- Müşteri ikinci ekran
- Mobil istemci + isteğe bağlı bulut Postgres
