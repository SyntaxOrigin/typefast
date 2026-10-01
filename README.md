# TypeFast (YazHızlı) — Genel Metin Genişletici ve Pano Geçmişi

> # ⚠️ GİZLİLİK UYARISI — LÜTFEN ÖNCE OKUYUN
>
> **Bu araç pano geçmişini ve şablonlarını ŞİFRELEMEZ.**
>
> `sablonlar.json`, `pano.json` ve `istatistik.json` dosyaları **düz metin**dir ve
> diske açık yazılır. `mask` komutu yalnız **ekranda gösterilen metni** maskeler;
> dosyadaki kayıt değişmez. Gizli mod (`--gizli`) bir gizlilik *kuralıdır,
> şifreleme değildir*.
>
> Bu nedenle **bu aracı gerçek parola, API anahtarı, kişisel kimlik numarası veya
> müşteri verisi ile kullanmayın.** Kullanacaksanız önce
> `clipboard temizle` çalıştırın ve diskte tam disk şifrelemesi kullanın.
> Depo dosyasının tamamını tek komutla silmek için `pano.json` dosyasını silmek
> yeterlidir.
>
> Davranış öğrenmesi, telemetri, ağ çağrısı ve otomatik güncelleme **yoktur**.

---

Genel metin (e-posta, rapor, destek yanıtı, günlük kayıt) için **kural tabanlı**,
**öngörülebilir** ve **şifresiz** bir metin genişletici. Aynı girdi her zaman aynı
çıktıyı verir; hangi kısayolun neden genişlediği `expand --gerekce` ile
görülebilir. Kazanç "hissedilen hız" değil, **sayıyla** raporlanır.

> **Bu araç neden SnipHub değil?** (MANIFEST karar D-011)
> TypeFast **genel metin** içindir ve programlama bilgisi gerektirmez. Bir özellik
> yalnızca "hangi dilde yazıldı" bilgisine bağlıysa kardeş proje
> **SnipHub**'ın kapsamındadır. Bu projede **sözdizimi denetleyicisi, `kod_dili`
> alanı ve dil bazlı parça deposu bilinçli olarak uygulanmamıştır.** Ayrımın
> test edilebilir kuralı MANIFEST'te yazılıdır.

## Özellikler

- **Şablon deposu** — her şablon `tetik`, `govde`, `kategori`, `aciklama`,
  `gizli` ve `kullanim` alanlarını taşır. Aynı tetik iki kez eklenemez
  (çakışma `add` anında reddedilir).
- **Genişleme motoru** — `{{tarih}}`, `{{saat}}`, `{{zaman}}`, `{{isim}}`,
  `{{bos}}` yerleşikleri; `{{anahtar|deger}}` tanımlı yer tutucuları;
  `{{>tetik}}` iç içe şablon çağrısı; derinlik sınırı (8) ve adım tavanı (200);
  kapanmamış `{{` için hata; kendini çağıran şablon için döngü hatası.
- **Kısayol çözümleme** — nokta, boşluk ve noktalama ile ayrılmış **kelime
  dizisi** (en fazla 4 kelime). Öncelik sırası: **en uzun eşleşme → tam sayı →
  en sık kullanım → belirsizlik hatası** (sessizce yanlış kısayol seçilmez).
- **Pano geçmişi** — sınırlı halka tampon (varsayılan 200 kayıt), tek öğe boyut
  sınırı (4096 karakter), en eski kayıdın otomatik düşürülmesi, "atlandı"
  sayacı ve büyük/küçük harf duyarsız arama.
- **Gizli mod** — `parola:`, `sifre=`, `api_key:`, `ghp_…`, `sk-…`, `AKIA…`,
  `Bearer …` ve yüksek entropili diziler kural tabanlı olarak tanınır; çıktı
  maskelenir, pano kaydı gizli işaretlenir ve aramada varsayılan olarak görünmez.
  `clipboard oku <id> --coz` ile geri döner.
- **Ölçülebilir kazanç** — `std::time::Instant` ile ölçülen gerçek süre ve gerçek
  tuş sayacı. Ölçüm yapılmamışsa `stats` **"OLCULMEDI"** yazar; sıfır uydurmaz.
- **Dışa/içe aktarma** — JSON paket ve sekmeyle ayrılmış düz metin liste.
  İkisi de gidiş-dönüşlüdür. **Pano geçmişi ve istatistik dışa aktarılmaz.**
- **Oturum kipi** — `--oturum` bayrağı hiçbir şeyi diske yazmaz.
- **Atomik yazma** — her yazma `*.tmp` + `fs::rename` ile yapılır.

## Kurulum

Gereksinimler: Rust **1.74+** (bu makinede `cargo 1.98.1`, `rustc 1.98.1` ile
derlendi) ve Windows'ta MinGW bağlantı programı.

```powershell
$env:PATH = "%USERPROFILE%\.cargo\bin;%USERPROFILE%\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;" + $env:PATH
cargo build --release
```

```text
    Finished `release` profile [optimized] target(s) in 19.07s
```

Çalıştırma:

```powershell
.\target\release\typefast.exe --help
```

```text
Genel metin icin ongorulebilir sablon genisletici, kisayol cozumleyici ve duz metin pano gecmisi.

Usage: typefast.exe [OPTIONS] <COMMAND>

Commands:
  add        Yeni sablon ekler
  list       Sablonlari listeler
  expand     Bir sablonu genisletir
  clipboard  Pano gecmisi islemleri
  search     Sablon ve pano gecmisinde arama yapar
  export     Sablonlari disa aktarir
  import     Sablonlari ice aktarir
  stats      Kazanc istatistigini gosterir
  mask       Metindeki sirlari maskeler
  conflicts  Ayni kelime dizisini paylasan tetikleri gosterir
  yollar     Depo dosyalarinin yollarini gosterir
  help       Print this message or the help of the given subcommand(s)

Options:
      --depo <YOL>  Depo dosyalarinin bulunacagi dizin
      --oturum      Yalnizca bellekte calis; hicbir seyi diske yazma
      --isim <AD>   `{{isim}}` yerlesiginin degeri
  -h, --help        Print help
  -V, --version     Print version
```

Sisteme kurma:

```powershell
cargo install --path .
```

Aşağıdaki tüm örnekler `ornek/` klasöründeki 19 şablonluk gerçek bir örnek
koleksiyonla çalıştırılmıştır:

```powershell
$env:TYPEFAST_DEPO = ".\ornek"
```

## Kullanım

### Şablon ekleme

```powershell
.\target\release\typefast.exe add ";yeniacilis" "Merhaba {{isim}}, talebinizi aldım." --kategori iletisim --aciklama "Yeni acilis sablonu"
```

```text
eklendi: ;yeniacilis (35 karakter, 1 adim dogrulandi)
```

Aynı tetik ikinci kez eklenmeye çalışılırsa **reddedilir**:

```powershell
.\target\release\typefast.exe add ";tesekkur" "Baska govde"
```

```text
typefast: ';tesekkur' zaten kullanılıyor (Merhaba, teşekkür ederim.)
cikis kodu: 1
```

### Listeleme

```powershell
.\target\release\typefast.exe list
```

```text
19 sablon
;tesekkur            43  (iletisim) Merhaba, teşekkür ederim.
;yol                 17  (iletisim) Size nasıl yardımcı olabilirim?
;kapatis             11  (iletisim) İyi çalışmalar dilerim.

Saygılarımla,
acilis               31  (parca) Sayın {{musteri|Kıymetli Müşteri}},

selam                29  (parca) {{>acilis}}

;selam               29  (iletisim) {{>selam}}
;kisa selam           5  (iletisim) {{>selam}}Size nasıl yardımcı olabilirim?
;imza                24  (iletisim) —
Gönderen: {{isim}}
Tarih: {{tarih}} {{saat}}
;not                  9  (kayit) NOT ({{zaman}}) — bu kayıt sistemde arşivlenmişt…
is parcasi           65  (destek) Talebinizi aldım, en kısa sürede size dönüş yapa…
bilgi talebi         18  (destek) Talebiniz için gerekli bilgiyi topluyorum. {{tar…
cozuldu              21  (destek) Talebiniz çözüldü; memnuniyetiniz bizim için öne…
;randevu             14  (randevu) {{tarih}} tarihinde {{saat}} saatinde randevunuz…
;toplanti             7  (toplanti) Toplantı notu ({{zaman}})
Katılımcılar: {{katili…
kayit-ekle            4  (sistem) Kayıt eklendi: {{kayit_no|Kimlik üretilmedi}}
;baglanti             3  [gizli] (sistem) postgresql://kullanici:OrnekSifre2026@localhost:…
;token                1  [gizli] (sistem) token: ghp_***REDACTED***
;sifre notu           0  [gizli] (sistem) parola: NotParola2026
Not: bu satır sadece örnek…
,toplanti             2  (toplanti) Toplantı tutanağı eklendi.
```

```powershell
.\target\release\typefast.exe list --en-cok 5
```

```text
5 sablon
is parcasi           65  (destek) Talebinizi aldım, en kısa sürede size dönüş yapa…
;tesekkur            43  (iletisim) Merhaba, teşekkür ederim.
acilis               31  (parca) Sayın {{musteri|Kıymetli Müşteri}},

;selam               29  (iletisim) {{>selam}}
selam                29  (parca) {{>acilis}}
```

```powershell
.\target\release\typefast.exe list --kategori destek
```

```text
3 sablon / destek
is parcasi           65  (destek) Talebinizi aldım, en kısa sürede size dönüş yapa…
bilgi talebi         18  (destek) Talebiniz için gerekli bilgiyi topluyorum. {{tar…
cozuldu              21  (destek) Talebiniz çözüldü; memnuniyetiniz bizim için öne…
```

### Genişletme

```powershell
.\target\release\typefast.exe --isim Ceren expand ";imza" --gerekce
```

```text
—
Gönderen: Ceren
Tarih: 2026-09-29 17:41:53
# cozum: tetik=;imza gerekce=tek-aday adim=3 derinlik=0 sure=9 us
# kazanc: gerekli=44 tus gercek=6 tus kazanc=38 (kazanc)
```

İç içe çağrı (derinlik 2): `;kisa selam` → `{{>selam}}` → `{{>acilis}}`.

```powershell
.\target\release\typefast.exe --isim Ceren expand ";kisa selam" --gerekce
```

```text
Sayın Kıymetli Müşteri,

Size nasıl yardımcı olabilirim?
# cozum: tetik=;kisa selam gerekce=tek-aday adim=3 derinlik=2 sure=14 us
# kazanc: gerekli=56 tus gercek=12 tus kazanc=44 (kazanc)
```

Kullanıcı değeri geçirme:

```powershell
.\target\release\typefast.exe --isim Ceren expand ";randevu" --deger randevu_no=RV-2026-4471
```

```text
2026-09-29 tarihinde 17:41:53 saatinde randevunuz alınmıştır.

Randevu no: RV-2026-4471
```

### Pano geçmişi

```powershell
.\target\release\typefast.exe clipboard ekle "Merhaba, teşekkür ederim."
```

```text
zaten vardi, basa tasindi: kayit #1
```

```powershell
.\target\release\typefast.exe clipboard ekle ("x" * 5000)
```

```text
atlandi (boyut siniri 4096 karakter); toplam atlanan: 1
```

```powershell
.\target\release\typefast.exe clipboard liste
```

```text
10 kayit (sinir 200), 1 atlandi
  10     87  2026-09-29 tarihinde 17:41:53 saatinde randevunuz alınmıştır.

Randevu no: RV-2026-4471
   9     56  Sayın Kıymetli Müşteri,

Size nasıl yardımcı olabilirim?
   8     44  —
Gönderen: Ceren
Tarih: 2026-09-29 17:41:53
   7     47  [gizli] **** (47 karakter)
   6     23  [gizli] **** (23 karakter)
   5     49  Talebinizi aldım, en kısa sürede dönüş yapacağım.
   1     25  Merhaba, teşekkür ederim.
   4     20  Sayın Ayşe Yılmaz,

   3     54  Talebinizi aldım, en kısa sürede size dönüş yapacağım.
   2     44  —
Gönderen: Ceren
Tarih: 2026-09-29 17:41:39
```

Gizli kayıt varsayılan olarak maskeli gelir; ozgun metin **yalnız** açıkça
istendiğinde döner:

```powershell
.\target\release\typefast.exe clipboard oku 6
```

```text
**** (23 karakter)
```

```powershell
.\target\release\typefast.exe clipboard oku 6 --coz
```

```text
parola: OrnekParola2026
```

```powershell
.\target\release\typefast.exe clipboard ara "Merhaba"
```

```text
1 sonuc
   1  Merhaba, teşekkür ederim.
```

### Gizli mod

```powershell
.\target\release\typefast.exe mask "parola: KirmiziKitap2026"
```

```text
parola: ****
# bulgu: 1 (kaynak metin DEGISMEDI; yalniz goruntulenen metin maskelendi)
```

```powershell
.\target\release\typefast.exe mask "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig" --tur
```

```text
bearer
```

Sır olmayan metinde **yanlış pozitif üretilmez**:

```powershell
.\target\release\typefast.exe mask "Toplanti 14:30'da baslayacak."
```

```text
Toplanti 14:30'da baslayacak.
# bulgu: 0 (kaynak metin DEGISMEDI; yalniz goruntulenen metin maskelendi)
```

### Çakışma denetimi

```powershell
.\target\release\typefast.exe conflicts
```

```text
belirsiz tetik yok
```

### Arama

```powershell
.\target\release\typefast.exe search ";tesekkur is parcasi"
```

```text
sablon: 2 eslesme
  @0  ;tesekkur (tek-aday)
  @10  is parcasi (tek-aday)
```

### Dışa / içe aktarma

> Not: Aşağıdaki çıktılarda örnek jeton `ghp_***REDACTED***` olarak
> kısaltılmıştır. `ornek/sablonlar.json` içindeki değer **uydurma** bir test
> vektörüdür (gerçek bir anahtar değildir) ve otomatik gizli taramayı geçmesi
> için `ornek/` klasöründe kesintisiz yazılmıştır.

```powershell
.\target\release\typefast.exe export --bicim metin
```

```text
# typefast sablon listesi v1
;tesekkur	iletisim	Günlük açılış cümlesi		43	Merhaba, teşekkür ederim.
;yol	iletisim	Soruyu takip eden açılış		17	Size nasıl yardımcı olabilirim?
;kapatis	iletisim	Kapanış paragrafı		11	İyi çalışmalar dilerim.\n\nSaygılarımla,
acilis	parca	Parça şablon: diğer şablonlar bunu çağırır		31	Sayın {{musteri|Kıymetli Müşteri}},\n
selam	parca	Parça şablon: açılış + boş satır		29	{{>acilis}}\n
;selam	iletisim	İç içe çağrı örneği: selam parçasını genişletir		29	{{>selam}}
;kisa selam	iletisim	En uzun eşleşme örneği		6	{{>selam}}Size nasıl yardımcı olabilirim?
;imza	iletisim	Yerleşik yer tutucu örneği		25	—\nGönderen: {{isim}}\nTarih: {{tarih}} {{saat}}
;not	kayit	ISO zaman damgalı not		9	NOT ({{zaman}}) — bu kayıt sistemde arşivlenmiştir.
is parcasi	destek	İki kelimelik tetik: en uzun eşleşme kuralı		65	Talebinizi aldım, en kısa sürede size dönüş yapacağım.
bilgi talebi	destek	Destek ekibi ara cevap metni		18	Talebiniz için gerekli bilgiyi topluyorum. {{tarih}} itibarıyla.
cozuldu	destek	Kapatma metni		21	Talebiniz çözüldü; memnuniyetiniz bizim için önemli.
;randevu	randevu	Tanımlı yer tutucu örneği (varsayılan değer)		15	{{tarih}} tarihinde {{saat}} saatinde randevunuz alınmıştır.\n\nRandevu no: {{randevu_no|Kimlik üretilmedi}}
;toplanti	toplanti	Çok satırlı tanımlı yer tutucu örneği		7	Toplantı notu ({{zaman}})\nKatılımcılar: {{katilimci|belirtilmedi}}\nGündem: {{gundem|belirlenecek}}
kayit-ekle	sistem	Tire içeren tek kelimelik tetik örneği		4	Kayıt eklendi: {{kayit_no|Kimlik üretilmedi}}
;baglanti	sistem	Gizli işaretli: bağlantı dizesi ekranda maskelenir	gizli	3	postgresql://kullanici:OrnekSifre2026@localhost:5432/veritabani
;token	sistem	Gizli işaretli: jeton kalıbı otomatik tanınır	gizli	1	token: ghp_***REDACTED***
;sifre notu	sistem	Parola kalıbı otomatik tanınır ve maskelenir	gizli	0	parola: NotParola2026\nNot: bu satır sadece örnek içindir.
,toplanti	toplanti	Virgül ön ekli tetik örneği		2	Toplantı tutanağı eklendi.
```

```powershell
.\target\release\typefast.exe export --bicim json
```

```text
{
  "surum": 1,
  "ureten": "typefast 0.1.0",
  "sablonlar": {
    "surum": 1,
    "sablonlar": [
      {
        "tetik": ";tesekkur",
        "govde": "Merhaba, teşekkür ederim.",
        "kategori": "iletisim",
        "aciklama": "Günlük açılış cümlesi",
        "gizli": false,
        "kullanim": 43
      },
```

```powershell
.\target\release\typefast.exe --depo C:\...\tmp export C:\...\tmp\paket.json
```

```text
yazildi: %USERPROFILE%\AppData\Local\Temp\opencode\tf-demo\paket.json (19 sablon, 4980 bayt)
```

```powershell
.\target\release\typefast.exe --depo C:\...\tmp import C:\...\tmp\paket.json
```

```text
bicim: json | eklendi: 19 | atlandi: 0
```

Aynı paket ikinci kez alınırsa çakışan kayıtlar atlanır:

```text
bicim: json | eklendi: 0 | atlandi: 19
```

`--degistir` mevcut depoyu **temizleyip** paketi yazar:

```text
bicim: json | eklendi: 19 | atlandi: 0
```

Biçim `otomatik` (varsayılan) sezgilenir; düz metin listesi de aynı şekilde
alınır:

```text
bicim: metin | eklendi: 19 | atlandi: 0
```

Dosya verilmezse **standart girdi** okunur (`cat paket.json | typefast import`).

### Kazanç istatistiği

```powershell
.\target\release\typefast.exe stats
```

```text
== typefast olcum raporu ==
tetik sayisi      : 19
pano kaydi        : 10
pano siniri       : 200
atlanan kayit     : 1
gizli isaretli kayit: 2
tus carpani       : 1.00
genisleme         : 7
kazanilan tus     : 269 (carpan 1.00 uygulanmis)
kazanilan karakter: 330
toplam sure       : 0.000065 sn
ortalama sure     : 0.000009 sn
en cok kazandiran:
  ;randevu         adet=1    kazanc=78
  ;imza            adet=2    kazanc=76
  ;kisa selam      adet=1    kazanc=44
  is parcasi       adet=1    kazanc=43
  ;tesekkur        adet=1    kazanc=15
```

**Ölçüm protokolü** (rapor `b08`'in "bu değer kullanıcı tarafından düzeltilebilir
olmalıdır" isteğine yanıt):

| Ölçüt | Tanım |
|---|---|
| `gerekli_tus` | Şablon yoksa gövde metnini elle yazmak için gereken tuş = gövde karakter sayısı |
| `gercek_tus` | Kısayolla basılan tuş = tetik karakter sayısı + 1 (genişletme tuşu) |
| `kazanc` | `gerekli_tus − gercek_tus`; negatifse zararlı ve öyle raporlanır |
| `toplam sure` | `std::time::Instant` ile ölçülen gerçek süre |

Varsayılan tuş çarpanı `1.00`'dür (bir karakter = bir tuş). Türkçe klavyede
`ğ`, `ü`, `ş` birden çok tuşla yazıldığı için **gerçek oran 1'in üzerindedir**;
`stats --tus-carpani 1.8` ile düzeltilebilir. Bu değer bir **varsayımdır**,
ölçüm değildir ve raporda böyle yazılıdır.

### Oturum kipi

```powershell
.\target\release\typefast.exe --oturum add ";gecici" "Bu kayit diske yazilmaz."
```

```text
eklendi: ;gecici (24 karakter, 0 adim dogrulandi)
```

```powershell
.\target\release\typefast.exe --depo C:\...\tmp clipboard liste --adet 3
```

```text
pano gecmisi bos
```

### Depo yolları

```powershell
.\target\release\typefast.exe yollar
```

```text
kok        : %USERPROFILE%\Desktop\Projeler\projects\25-typefast\ornek
sablonlar  : %USERPROFILE%\Desktop\Projeler\projects\25-typefast\ornek\sablonlar.json
pano       : %USERPROFILE%\Desktop\Projeler\projects\25-typefast\ornek\pano.json
istatistik : %USERPROFILE%\Desktop\Projeler\projects\25-typefast\ornek\istatistik.json
oturum     : kapali
```

## Test

```powershell
cargo test
```

```text
     Running unittests src\lib.rs
running 189 tests
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests\entegrasyon.rs
running 32 tests
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

**test sonucu: okunan 221; başarısız 0** (189 birim + 32 entegrasyon).

Diğer kalite kapıları:

```powershell
cargo build --release   # Finished `release` profile [optimized]
cargo clippy --all-targets -- -D warnings   # Finished `dev` profile
cargo fmt --check       # çıktı yok, aykırılık yok
```

Kapsanan konular:

| Konu | Nerede |
|---|---|
| Genişleme söz dizimi (yerleşik, tanımlı yer tutucu, kapanmamış `{{`) | `genislet::tests` |
| İç içe şablon çağrısı ve derinlik sınırı | `genislet`, `entegrasyon::derinlik_siniri_varsayilan_degerde_sekiz` |
| Döngü koruması (kendini çağırma, iki düğümlü döngü) | `genislet::tests` |
| Adım tavanı (eşitte geçer, bir fazlasında hata) | `genislet::tests` |
| Kısayol çözümleme: tam / en uzun / en sık / belirsiz / çakışma | `cozumle::tests`, `entegrasyon::cozumleyici_*` |
| Tetik sözdizimi (kelime sınırı, tire, ön ek, ayraç) | `cozumle::tests` |
| Pano halkası sınırı, eski kayıt düşürme, boyut sınırı, tekrar taşıma | `pano::tests`, `entegrasyon::pano_*` |
| Gizli mod maskeleme: parola / token / anahtar / bearer / yüksek entropi | `gizli::tests` |
| Maskeli kaydın geri dönüşü ve aramadan gizlenmesi | `entegrasyon::gizli_kayit_*` |
| JSON şema gidiş-dönüşü ve bozuk depo | `depo::tests`, `aktarma::tests`, `entegrasyon::bozuk_*` |
| Atomik yazma (geçici dosya bırakmama, güncelleme) | `entegrasyon::atomik_yazma_*` |
| Kazanç hesabı (pozitif/zararlı/çarpanlı) | `kazanc::tests`, `entegrasyon::kazanc_*` |
| Stat raporu ve "ölçülmedi" durumu | `entegrasyon::olculmedigi_zaman_*` |
| Dışa/içe aktarma: JSON ve düz metin gidiş-dönüşü | `aktarma::tests`, `entegrasyon::*_aktarma_*` |
| Unicode tetikleyici (emoji, CJK, Türkçe büyük harf) | `cozumle::tests`, `entegrasyon::turkce_ve_unicode_*` |

Testlerde sabit zaman vektörü `2026-09-29T14:05:09Z` (epoch `1790690709`) kullanılır;
`SystemTime::now()` hiçbir testin kararında yer almaz.

## Proje Yapısı

```text
25-typefast/
├── Cargo.toml
├── Cargo.lock
├── LICENSE.txt
├── README.md
├── .gitignore
├── src/
│   ├── lib.rs            çekirdek kütüphane, modül haritası, crate lint'leri
│   ├── main.rs           ikili giriş noktası (yalnız clap ayrıştırma + çıkış kodu)
│   ├── hata.rs           tek hata tipi, elle Display/Error uygulaması
│   ├── sablon.rs         şablon varlığı, depo, çakışma denetimi, kullanım sayacı
│   ├── cozumle.rs        tetik sözdizimi, metin taraması, belirsizlik çözümü
│   ├── genislet.rs       {{ ... }} genişletme motoru, tarih dönüşümü, döngü koruması
│   ├── gizli.rs          sır kalıbı tanıma ve maskeleme
│   ├── pano.rs           sınırlı pano geçmişi halka tamponu
│   ├── kazanc.rs         tuş sayacı tabanlı ölçüm ve kalıcı istatistik
│   ├── depo.rs           JSON yükleme/kaydetme, atomik yazma
│   ├── aktarma.rs        JSON paket ve düz metin liste dışa/içe aktarma
│   └── cli.rs            clap alt komutları ve çıktı biçimleri
├── tests/
│   ├── entegrasyon.rs    uçtan uca dosya üzerinden akışlar
│   └── yardimci/mod.rs   Drop ile temizleyen geçici dizin yardımcısı
└── ornek/
    ├── sablonlar.json    19 şablonluk gerçek örnek koleksiyon
    ├── pano.json         gerçek `clipboard ekle` çalıştırmalarından üretildi
    └── istatistik.json   gerçek `expand` çalıştırmalarından üretildi
```

Satır sayıları (`.rs`, `target/` hariç): **5.252**.

## Yapılandırma

| Ayar | Tür | Varsayılan | Etki |
|---|---|---|---|
| `--depo <YOL>` | yol | yok | Depo dizini. Verilmezse `TYPEFAST_DEPO`, yoksa çalıştırılabilirin yanı. |
| `TYPEFAST_DEPO` | ortam | yok | `--depo` yoksa kullanılır. |
| `--oturum` | bayrak | kapalı | Hiçbir şey diske yazılmaz; pano geçmişi ve sayaçlar güncellenmez. |
| `--isim <AD>` | metin | `Kullanici` | `{{isim}}` yerleşiğinin değeri. |
| `export/import --bicim` | `json`/`metin`/`otomatik` | `json` / `otomatik` | Aktarım biçimi. `otomatik` içerikten sezgiler. |
| `import --degistir` | bayrak | kapalı | Var olan depo temizlenip paketin tamamı yazılır. |
| `stats --tus-carpani <K>` | ondalık | `1.0` | Bir karakterin kaç tuş vuruşu sayıldığı. |
| `stats --en-cok <N>` | sayı | `5` | En çok kazandıran tetik sayısı. |
| `clipboard liste --adet <N>` | sayı | `20` | Listelenecek kayıt sayısı. |
| `clipboard liste --gizliler` | bayrak | kapalı | Gizli kayıtlar da (maskeli) listelenir. |
| `expand --deger A=B` | liste | yok | Yer tutucu değerleri; tekrar edilebilir. |
| `add --kategori/--aciklama/--gizli` | bayrak/metni | yok | Şablon alanları. |

### Dosya biçimi

```json
{
  "surum": 1,
  "sablonlar": [
    {
      "tetik": ";tesekkur",
      "govde": "Merhaba, teşekkür ederim.",
      "kategori": "iletisim",
      "aciklama": "Günlük açılış cümlesi",
      "gizli": false,
      "kullanim": 43
    }
  ]
}
```

`pano.json`: `surum`, `kapasite`, `maks_boyut`, `sonraki_id`, `atlanan`, `kayitlar[]`.
`istatistik.json`: `surum`, `genisletme`, `kazanilan_tus`, `kazanilan_karakter`,
`gecen_sn`, `tus_carpani`, `tetikler{}`.

Eksik alanlar varsayılan değerle doldurulur; sürüm damgası bilinmeyen ya da daha
yüksek bir belge reddedilir. Bozuk JSON `typefast: '<yol>' bozuk: <ayrıntı>` hatası
verir.

### `#[allow]` listesi

`#![forbid(unsafe_code)]` **korunur**; projede hiçbir `unsafe` yoktur ve platform
FFI'si kullanılmaz. Tek `#[allow]` grubu testlerdedir:
`#[allow(clippy::unwrap_used, clippy::expect_used)]` — testlerde `expect`/`.err()`
kullanımının gerekçesi modül başına yazılıdır ve `WORKER_CONTRACT.md` § 4.2
"yalnızca testlerde ve burada da gerekçeyle kullanılabilir" kuralına uyar. Üretim
kodunda bu lint'ler açıktır.

## Bilinen Sınırlamalar

**Bu bölüm kasıtlı olarak rahatsız edicidir; ürünün gerçek sınırlarıdır.**

1. **Şifreleme yoktur.** Pano geçmişi, şablonlar ve istatistik **düz metin**
   JSON'dur. `mask` yalnız ekran çıktısını maskeler. Raporun "şifreli pano
   geçmişi" vaadi `MANIFEST` kart 25 tarafından kapsamdan çıkarıldı; bu,
   ürünün en görünür eksikliğidir.
2. **Pano dinlemez, global kısayol kaydetmez.** Panoya erişim `clipboard ekle`
   ile elle yapılır. Raporun "yaz, kısayola bas, yapıştır" akışı çalışmaz.
   Gerekçe: bu yollar `unsafe` platform FFI'si gerektirir ve `#![forbid(unsafe_code)]`
   ile bağdaşmaz.
3. **Davranış öğrenmesi uygulanmamıştır — karar ve gerekçe.** Rapor bunu `v1`
   özelliği olarak listeler. Burada **hiç eklenmemiştir**; kısmi bir "opt-in"
   sürümü de eklenmedi. Gerekçe: (a) kazancın ve gizliliğin ölçülebilir
   kalması ancak kural tabanlı davranışla mümkündür — öğrenme katmanı "aynı girdi
   aynı çıktı" garantisini bozar; (b) öğrenme verisi kullanıcının yazım
   alışkanlığını diske yazar ve bu, MANIFEST'te ertelenen bir kalemdir;
   (c) sıfır davranış, `stats` çıktısının denetlenebilir kalmasını sağlar.
   Karar koda ve `ornek/` verisine yansımıştır: hiçbir yerde frekans tablosu yoktur.
4. **Sözdizimi denetleyicisi, `kod_dili` ve dil bazlı parça deposu YOKTUR**
   (MANIFEST karar D-011, yukarıdaki "Bu araç neden SnipHub değil?" notu).
5. **Tetik eşleşmesi kelime sınırına duyarlıdır ama bağlama duyarsızdır.**
   Hangi uygulamada yazıldığı bilinmez. Ön ek (`, ; : . ! ? #`) kullanmak
   çoğu yanlış pozitifi önler; örnek koleksiyondaki `acilis` ve `selam` gibi
   ön eksiz "parça" tetikleri normal metinde de genişler ve bilinçli bir
   tercihtir.
6. **Zaman dilimi daima UTC'dir.** `chrono`/`time` bağımlılıkları yasak
   (`WORKER_CONTRACT.md` § 3.2-F); tarih dönüşümü elle, Hinnant'ın kamu malı
   algoritmasıyla yapılır. Yerel saat veya yaz saati desteği yoktur.
7. **Türkçe küçük harf indirgemesi ASCII'leştirir:** `I`, `İ`, `ı` → `i`.
   `ş` ve `ç` **korunur**. Bu yüzden `tesekkur` ile `teşekkür` eşleşmez; bu,
   Türkçe yazımı bozmamak için bilinçlidir ama kullanıcıyı şaşırtabilir.
8. **Tuş sayısı bir varsayımdır**, ölçüm değildir (bkz. `## Kullanım`).
9. **Sır kalıbı tanıyıcı kural tabanlıdır.** Kalıba uymayan bir sır maskelenmez
   (yanlış negatif). Kapsanan kalıplar: 19 anahtar sözcük + 6 jeton ön eki +
   `Bearer` + 24 karakter üzeri yüksek entropili dizi. Maskeleme **yalnız
   gösterim içindir**; kaynak metin değişmez.
10. **Pano geçmişinde zaman damgası `0`'dır.** Kayıt ne zaman eklendiyse o
    an kaydedilir, ama tarih bilgisi saklanmaz (`std::time` tabanlı zaman
    damgası `depo.rs`te bilinçli olarak kullanılmıyor; pano geçmişi için tarih
    bir güvenlik riski olarak değerlendirildi). `genisletme` süresi yine de
    `Instant` ile ölçülür.
11. **`config.toml` yoktur.** Rapor yanındaki `config.toml`'i öneriyordu; TOML
    ayrıştırıcı izinli bağımlılık listesinde yok. Ayarlar bayrak ve ortam
    değişkeni ile verilir.
12. **İkili imzası ve paketleyici yoktur** (rapor `b09` riski). İmzasız tek
    dosya ikilileri bazı antivirüslerde uyarı üretebilir.
13. **Yalnızca Windows'ta çalıştırılmıştır.** Kod platformdan bağımsız saf
    Rust'tur; macOS/Linux'ta derlenmesi beklenir ama **ölçülmemiştir**.

### Ertelenenler (MANIFEST kart 25)

Şifreli pano geçmişi · davranış öğrenmesi · sürükle-bırak / URL'den aktarma ·
kısayol paketi dışa/içe aktarmanın bütünlük özeti ile doğrulanması · uygulama
imzaları ve veri taraması · pano ve global kısayol izleme.

## Gelecek Geliştirmeler

- **Bütünlük özeti ile dışa/içe aktarma** — rapor `b05` v2 özelliği; paket
  özeti tutmazsa içe aktarma durmalı ve hangi kaydın değiştiği bildirilmeli.
- **Yerel saat desteği** — TZ ortam değişkenini okuyan, `std::time` ile kalan
  bir dönüşüm (kütüphane eklemeden).
- **Yol bağımsız taşınabilir kip** — yapılandırmayı ikilinin yanına taşımak,
  salt-okunur çalıştırma testi.
- **Şablon şemaları** — kategoriye göre zorunlu alan kümeleri.
- **Sıralı odaklanma** — rapor `b05` "sıralı odaklanma, atla ve geri dön"
  özelliği; terminal arayüzünde odak sırası bir dosya olarak saklanabilir.
- **Sürükle-bırak / adresten tetik önerisi** — yerel ayrıştırma ile, ağ gönderimi
  olmadan.

## Troubleshooting

**1. `typefast: 'X' zaten kullanılıyor (...)` — `add` başarısız**

Belirti: yeni tetik eklenmiyor, çıkış kodu `1`. Neden: depoda aynı tetik zaten var
(`add`, çakışmayı sessizce ezmek yerine reddeder; MANIFEST rapor `b03` — S5).
Çözüm: `typefast list` ile mevcut tetiği görün, ya da farklı bir ön ek
kullanın (ör. `,tesekkur`).

**2. `typefast: 'X' bozuk: expected value at line 1 column 12`**

Belirti: hiçbir komut çalışmıyor. Neden: `sablonlar.json` elle bozulmuş ya da
yarım yazılmış (normalde atomik yazma bunu engeller, ama dosya elle düzenlenmiş
olabilir). Çözüm: yedeğiniz varsa geri yükleyin; yoksa dosyayı silin — depo boş
kurulur ve `typefast import paket.json` ile geri gelir.

**3. `typefast: kapanmamış '{{' (bayt N)`**

Belirti: `expand` hata veriyor. Neden: gövdede açılmış ama kapatılmamış `{{`
var; araç bunu sessizce düz metin sanmaz. Çözüm: `typefast list --ayrinti` ile
gövdeyi görün ve eksik `}}` ekleyin.

**4. Genişleme çalışıyor ama tetik yazıldığı yerde genişlemiyor**

Belirti: `typefast expand ";tesekkur"` çalışıyor, ama dokümanda `;tesekkur`
yazınca olmuyor. Nedenlar: (a) tetikte ön ek var (`;tesekkur`) ve dokümanda
`;tesekkur` **tam olarak** yazılmamış — ön ek zorunludur; (b) kelime içinde
yazılmış (`xx;tesekkur` gibi değil, `tesekkur` bir kelimenin başında olmalı);
(c) Unicode bir karakterden sonra kayma oluyorsa hata mesajını okuyun.
Çözüm: ön eki kaldırıp ön ek olmayan `tesekkur` şablonu ekleyin.

**5. `stats` çıktısında `OLCULMEDI` görünüyor**

Belirti: kazanç satırları yok. Neden: henüz hiç `expand` çalıştırılmamış — araç
sıfır uydurmaz, ölçülmemişse ölçülmedi der. Çözüm: `typefast expand ";tesekkur`
çalıştırıp `typefast stats` deyin. `--oturum` kipinde yapılan genişletmeler
**sayılmaz** (diske yazılmadıkları için).

## Atıflar

- **Howard Hinnant — "chrono-compatible Low-Level Date Algorithms"** —
  `gun_sayisindan_tarihe` fonksiyonunun kaynak algoritması (kamu malı),
  <https://howardhinnant.github.io/date_algorithms.html>
- **Rust standart kütüphane — `std::time`** — tarih/saat için yalnız izinli kaynak;
  `chrono` ve `time` bağımlılıkları yasaktır,
  <https://doc.rust-lang.org/std/time/>
- **Rust standart kütüphane — `std::collections::VecDeque`** — pano halkası,
  <https://doc.rust-lang.org/std/collections/struct.VecDeque.html>
- **Rust edition rehberi (2021)** — <https://doc.rust-lang.org/edition-guide/edition-2021/>
- **serde** — <https://serde.rs/> · **serde_json** — <https://github.com/serde-rs/json>
- **clap** — <https://docs.rs/clap/>
- **Espanso** — kural tabanlı metin genişletici referansı (rapor `b04`),
  <https://espanso.org/>
- **OWASP Credential Storage Cheat Sheet** — gizli modun kapsam dışı bırakılan
  kısıtları için, <https://cheatsheetseries.owasp.org/cheatsheets/Credential_Storage_Cheat_Sheet.html>
- **Fikir raporunun kendisi (iç tasarım kaynağı, URL değil yerel yol):**
  `%USERPROFILE%\Desktop\Fikirler\25-yaz-hizli-metin-genisletici.html` ve
  `%USERPROFILE%\Desktop\Projeler\MANIFEST.md` (kart 25).

Doğrudan kopyalanan üçüncü taraf kod yoktur.

## Üretim Atfı

Bu depo **OpenCode** ajanı tarafından, **`space-bunny-free`** modeli
(`opencode/space-bunny-free`) kullanılarak üretilmiştir.

- **Arac:** OpenCode
- **Model:** `opencode/space-bunny-free` (Space Bunny Free)
- **Tür:** Rust, `cargo build` / `cargo test` ile üretilmiş ve doğrulanmıştır.

Kaynak kod, testler ve dokümantasyon bu model tarafından yazılmıştır. İnsan
katkısı: gereksinim tanımı, kabul ölçütleri ve son kontroller.

## Lisans

**MIT** lisansı. Tam metin için [`LICENSE.txt`](LICENSE.txt) dosyasına bakın.
