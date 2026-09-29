//! Kısayol tetiği çözümlemesi.
//!
//! Raporun (`b03` — S5, `b07`) çekirdek kuralı: **çakışan tetiklerde sessizce
//! yanlış kısayol kullanılmaz.** Bu modül metni kelime dizisine ayırır, her
//! konumda eşleşen tetikleri toplar ve şu öncelik sırasıyla karar verir:
//!
//! 1. **En uzun eşleşme** — daha çok kelime kaplayan tetik kazanır.
//! 2. **Tam sayı** — girdinin tamamını kaplayan tetik doğrudan kazanır.
//! 3. **En sık kullanım** — kullanım sayacı büyük olan kazanır.
//! 4. Hâlâ eşitlik varsa **belirsizlik hatası** döner; adaylar kullanıcıya
//!    liste hâlinde gösterilir.
//!
//! Tetik sözdizimi: isteğe bağlı bir **ön ek ayracı** (`, ; : . ! ?` gibi bir
//! noktalama işareti) ve ardından **kelimelerden** oluşan bir dizi. Kelimeler
//! boşluk veya noktalama ile ayrılır; tire, iki harfin arasında kaldığında
//! kelimenin içinde sayılır (`kayit-ekle` tek kelimedir). Unicode kabul edilir,
//! böylece Türkçe tetikler de çalışır.

use crate::hata::{Sonuc, TypeFastHata};
use crate::sablon::{Sablon, SablonDeposu};

/// Bir tetikte kabul edilen en fazla kelime sayısı.
///
/// Sınır, çözümleyicinin n² büyümesini ve kullanıcının yazım hatasıyla üç
/// kelimelik bir tetik yazmasını engeller.
pub const EN_FAZLA_KELIME: usize = 4;

/// Tetik başında kabul edilen ön ek ayraçları.
pub const ONEK_AYRACLAR: [char; 7] = [',', ';', ':', '.', '!', '?', '#'];

/// Bir kelimeyi sonlandıran (içine girmeyen) karakterler.
const AYRACLAR: [char; 27] = [
    ' ', '\t', '\n', '\r', '\u{0B}', '\u{0C}', '.', ',', ';', ':', '!', '?', '"', '\'', '(', ')',
    '[', ']', '{', '}', '<', '>', '/', '\\', '*', '@', '#',
];

/// Bir karakterin ayraç olup olmadığını söyler.
fn ayrac_mi(karakter: char) -> bool {
    AYRACLAR.contains(&karakter)
}

/// Bir karakterin "harf ya da rakam" grubunda olup olmadığını söyler.
///
/// Türkçe'ye uygun büyük/küçük harf indirgeme.
///
/// `to_lowercase()` Türkçe'de `I` → `i̇` (i + birleşen nokta) üretir ve iki
/// kod noktasıyla sonuçlanır; bu tetik karşılaştırmasını bozduğu için dört
/// Türkçe harf elle ele alınır.
pub fn kucult_harf(metin: &str) -> String {
    let mut sonuc = String::with_capacity(metin.len());
    for karakter in metin.chars() {
        match karakter {
            'I' | 'İ' | 'ı' => sonuc.push('i'),
            _ => sonuc.extend(karakter.to_lowercase()),
        }
    }
    sonuc
}

/// Ayrıştırılmış ve doğrulanmış kısayol tetiği.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tetik {
    /// Tetiğin başında aranan noktalama ön eki (`None` ise ön ek aranmaz).
    pub onek: Option<char>,
    /// Küçültülmüş kelime listesi.
    pub kelimeler: Vec<String>,
    /// Normalleştirilmiş tam yazım; hata mesajlarında ve karşılaştırmada kullanılır.
    pub duz: String,
}

impl Tetik {
    /// Tetik yazımını ayrıştırır ve doğrular.
    ///
    /// # Hatalar
    ///
    /// Tetik boşsa, yalnız ayraçlardan oluşuyorsa, kelime sınırı aşılırsa veya
    /// ön ek ayracı geçersizse [`TypeFastHata::GecersizTetik`] döner.
    pub fn ayikla(yazim: &str) -> Sonuc<Self> {
        let kirp = yazim.trim();
        if kirp.is_empty() {
            return Err(TypeFastHata::GecersizTetik {
                tetik: yazim.to_string(),
                sebep: "tetik boş olamaz".to_string(),
            });
        }

        let karakterler: Vec<char> = kirp.chars().collect();
        let mut onek = None;
        let mut baslangic = 0usize;
        if ONEK_AYRACLAR.contains(&karakterler[0]) {
            onek = Some(karakterler[0]);
            baslangic = 1;
        }

        let kalan: String = karakterler[baslangic..].iter().collect();
        let kelimeler: Vec<String> = kalan
            .split(ayrac_mi)
            .filter(|k| !k.is_empty())
            .map(kucult_harf)
            .collect();

        if kelimeler.is_empty() {
            return Err(TypeFastHata::GecersizTetik {
                tetik: yazim.to_string(),
                sebep: "ön ekin ardından en az bir kelime olmalı".to_string(),
            });
        }
        if kelimeler.len() > EN_FAZLA_KELIME {
            return Err(TypeFastHata::GecersizTetik {
                tetik: yazim.to_string(),
                sebep: format!("en fazla {EN_FAZLA_KELIME} kelime kullanılabilir"),
            });
        }
        if kelimeler
            .iter()
            .any(|k| k.contains('-') && (k.starts_with('-') || k.ends_with('-')))
        {
            return Err(TypeFastHata::GecersizTetik {
                tetik: yazim.to_string(),
                sebep: "kelime tire ile başlayıp bitemez".to_string(),
            });
        }

        let duz = match onek {
            Some(isaret) => {
                let birlestirilmis = kelimeler.join(" ");
                format!("{isaret}{birlestirilmis}")
            }
            None => kelimeler.join(" "),
        };
        Ok(Self {
            onek,
            kelimeler,
            duz,
        })
    }

    /// Tetiğin kelime sayısı; öncelik karşılaştırmasının birincil ölçütüdür.
    pub fn kelime_sayisi(&self) -> usize {
        self.kelimeler.len()
    }
}

/// Metinden bir kelime okur; bulunamazsa `None` döner.
///
/// # Güvenlik
///
/// `konum` her zaman `metin` karakter uzunluğundan küçük olmalıdır; çağıran
/// taraf bunu döngü koşulunda garanti eder. Fonksiyon ayrıca `char_indices`
/// ile sınırı yeniden doğrular, bu yüzden taşma koşulunda sessizce boş
/// metin dönmek yerine `None` döner.
fn kelime_oku(metin: &str, konum: usize) -> Option<(String, usize, usize)> {
    if konum >= metin.len() {
        return None;
    }
    let baytlar = metin.as_bytes();
    let mut baslangic = konum;
    while baslangic < baytlar.len() && is_ayrac_bayt(baytlar[baslangic]) {
        baslangic += 1;
    }
    if baslangic >= baytlar.len() {
        return None;
    }
    let mut bitis = baslangic;
    while bitis < baytlar.len() && !is_ayrac_bayt(baytlar[bitis]) {
        bitis += 1;
    }
    let parca = metin.get(baslangic..bitis)?;
    let sozluk = parca.trim_matches(|c: char| c == '-' || c == '\'' || c == '’');
    if sozluk.is_empty() {
        return None;
    }
    Some((kucult_harf(sozluk), baslangic, bitis))
}

/// Bir baytın ayraç olup olmadığını, UTF-8 sınırına takılmadan belirler.
///
/// Çok baytlı bir karakterin içindeki baytlar `>= 0x80` olduğundan ASCII ayraç
/// testine takılmaz; böylece kaydırma sırasında geçersiz dilim oluşmaz.
fn is_ayrac_bayt(bayt: u8) -> bool {
    bayt < 0x80 && AYRACLAR.iter().any(|c| *c as u8 == bayt)
}

/// Bir eşleşmenin metindeki yerini ve hangi tetiğe ait olduğunu taşır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eslesme {
    /// Eşleşen tetiğin depodaki yazımı.
    pub tetik: String,
    /// Eşleşmenin başladığı bayt ofseti.
    pub baslangic: usize,
    /// Eşleşmenin bittiği bayt ofseti (sonlu).
    pub bitis: usize,
    /// Kapsanan kelime sayısı.
    pub kelime_sayisi: usize,
    /// Çözümde kullanılan ölçüt; belirsizlik gerekçesi olarak raporlanır.
    pub gerekce: CozumGerekcesi,
}

/// Çözümün hangi ölçüte dayandığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CozumGerekcesi {
    /// Tek aday vardı; başka ölçüt gerekmedi.
    Tek,
    /// Birden çok aday vardı, en uzun kelime dizisi kazandı.
    EnUzun,
    /// Girdinin tamamını kaplayan tetik kazandı.
    TamSayi,
    /// Kullanım sayacı karar verdi.
    EnSikKullanim,
}

/// Metin içinde çözülen tetik ve kalan metin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cozum {
    /// Eşleşen tetiğin depodaki yazımı.
    pub tetik: String,
    /// Eşleşmenin metindeki yeri.
    pub baslangic: usize,
    /// Eşleşmenin metindeki sonu.
    pub bitis: usize,
    /// Eşleşme metninde görünmüyorsa tetiğin depodaki yazımı, aksi hâlde
    /// metinden alınan yazımı döner.
    pub tetik_yazimi: String,
    /// Çözüm gerekçesi.
    pub gerekce: CozumGerekcesi,
}

/// Tetik listesine karşı çözümleyici kurar.
pub struct Cozumleyici<'a> {
    tetikler: Vec<(Tetik, &'a Sablon)>,
}

impl<'a> Cozumleyici<'a> {
    /// Depodaki tüm şablonlardan bir çözümleyici kurar.
    ///
    /// Sözdizimsel olarak hatalı tetikler sessizce atlanır; bunlar zaten
    /// [`crate::sablon::Sablon::tetigi_cozumle`] ile eklenirken reddedilmiştir.
    pub fn yeni(depo: &'a SablonDeposu) -> Self {
        let mut tetikler: Vec<(Tetik, &'a Sablon)> = Vec::with_capacity(depo.uzunluk());
        for sablon in depo.tumune() {
            if let Ok(tetik) = Tetik::ayikla(&sablon.tetik) {
                tetikler.push((tetik, sablon));
            }
        }
        Self { tetikler }
    }

    /// Çözümleyicideki tetik sayısı.
    pub fn uzunluk(&self) -> usize {
        self.tetikler.len()
    }

    /// `konum` baytından başlayarak eşleşen **tüm** tetikleri toplar.
    fn adaylar(&self, metin: &str, konum: usize) -> Vec<Eslesme> {
        let mut bulunan = Vec::new();
        for (tetik, sablon) in &self.tetikler {
            if let Some((baslangic, bitis)) = eslesme_baytlari(tetik, metin, konum) {
                bulunan.push(Eslesme {
                    tetik: sablon.tetik.clone(),
                    baslangic,
                    bitis,
                    kelime_sayisi: tetik.kelime_sayisi(),
                    gerekce: CozumGerekcesi::Tek,
                });
            }
        }
        bulunan
    }

    /// `metin` içindeki ilk eşleşmeyi çözer.
    ///
    /// Arama soldan sağa, her konumda en uzun kelime dizisi önce olmak üzere
    /// ilerler. Başlangıçta bulunan eşleşme varsa çözülür.
    ///
    /// # Hatalar
    ///
    /// Hiçbir tetik eşleşmezse [`TypeFastHata::BilinmeyenSablon`], eşit
    /// öncelikli birden fazla aday kalırsa
    /// [`TypeFastHata::CozumBelirsiz`] döner.
    pub fn coz(&self, metin: &str) -> Sonuc<Cozum> {
        let mut konum = 0usize;
        while konum <= metin.len() {
            if let Some(cozum) = self.cozum_aday(metin, konum)? {
                return Ok(cozum);
            }
            konum = ilerleme_adimi(metin, konum);
        }
        Err(TypeFastHata::BilinmeyenSablon {
            tetik: metin.to_string(),
        })
    }

    /// `konum` baytından başlayarak eşleşme varsa çözer.
    fn cozum_aday(&self, metin: &str, konum: usize) -> Sonuc<Option<Cozum>> {
        let adaylar = self.adaylar(metin, konum);
        if adaylar.is_empty() {
            return Ok(None);
        }

        // 1. En uzun kelime dizisi.
        let aday_sayisi = adaylar.len();
        let en_uzun = adaylar.iter().map(|a| a.kelime_sayisi).max().unwrap_or(0);
        let mut daralan: Vec<Eslesme> = adaylar
            .into_iter()
            .filter(|a| a.kelime_sayisi == en_uzun)
            .collect();
        if daralan.len() == 1 {
            let gerekce = if aday_sayisi > 1 {
                CozumGerekcesi::EnUzun
            } else {
                CozumGerekcesi::Tek
            };
            return Ok(Some(cozum_metnine(&daralan[0], metin, gerekce)));
        }
        // 2. Tam sayı: metnin tamamını kaplayan aday.
        if daralan.len() > 1 {
            let tam: Vec<&Eslesme> = daralan
                .iter()
                .filter(|a| a.baslangic == 0 && a.bitis == metin.len())
                .collect();
            if tam.len() == 1 {
                let secilen = tam[0].clone();
                return Ok(Some(cozum_metnine(
                    &secilen,
                    metin,
                    CozumGerekcesi::TamSayi,
                )));
            }
        }

        // 3. En sık kullanım.
        if daralan.len() > 1 {
            let kullanimlar: Vec<u64> = daralan
                .iter()
                .map(|a| {
                    self.tetikler
                        .iter()
                        .find(|(_, s)| s.tetik == a.tetik)
                        .map(|(_, s)| s.kullanim)
                        .unwrap_or(0)
                })
                .collect();
            let en_yuksek = kullanimlar.iter().copied().max().unwrap_or(0);
            let kalan: Vec<Eslesme> = daralan
                .iter()
                .zip(kullanimlar.iter())
                .filter(|(_, k)| **k == en_yuksek)
                .map(|(a, _)| a.clone())
                .collect();
            if kalan.len() == 1 {
                let secilen = &kalan[0];
                return Ok(Some(cozum_metnine(
                    secilen,
                    metin,
                    CozumGerekcesi::EnSikKullanim,
                )));
            }
            daralan = kalan;
        }

        // 4. Hâlâ eşitlik: belirsizlik. Sessizce seçim yapılmaz.
        let aday_yazimlar: Vec<String> = daralan.iter().map(|a| a.tetik.clone()).collect();
        Err(TypeFastHata::CozumBelirsiz {
            metin: metin.to_string(),
            adaylar: aday_yazimlar,
        })
    }

    /// Metindeki **tüm** eşleşmeleri soldan sağa döner.
    ///
    /// Eşleşmeler çakışamaz: bir eşleşme bulunduğunda tarama o eşleşmenin
    /// bittiği bayttan devam eder.
    pub fn tum_eslesmeler(&self, metin: &str) -> Vec<Cozum> {
        let mut liste = Vec::new();
        let mut konum = 0usize;
        while konum <= metin.len() {
            match self.cozum_aday(metin, konum) {
                Ok(Some(cozum)) => {
                    let sonraki = cozum.bitis.max(ilerleme_adimi(metin, konum));
                    liste.push(cozum);
                    konum = sonraki;
                }
                Ok(None) => konum = ilerleme_adimi(metin, konum),
                // Belirsizlikte tarama durur: sessizce sonraki konuma
                // gecmek yanlis sablonu secmek demektir.
                Err(_) => break,
            }
        }
        liste
    }
}

/// `konum`'dan bir sonraki denenecek bayta geçer.
///
/// Geçersiz UTF-8 sınırına basılmasın diye her adımda bir bayt ilerlenir;
/// ardından metin kırpılarak yeni bir karakter sınırına oturtulur.
fn ilerleme_adimi(metin: &str, konum: usize) -> usize {
    let mut hedef = konum.saturating_add(1);
    while hedef < metin.len() && !metin.is_char_boundary(hedef) {
        hedef += 1;
    }
    hedef
}

fn cozum_metnine(eslesme: &Eslesme, metin: &str, gerekce: CozumGerekcesi) -> Cozum {
    let tetik_yazimi = metin
        .get(eslesme.baslangic..eslesme.bitis)
        .map(kucult_harf)
        .unwrap_or_else(|| eslesme.tetik.to_lowercase());
    Cozum {
        tetik: eslesme.tetik.clone(),
        baslangic: eslesme.baslangic,
        bitis: eslesme.bitis,
        tetik_yazimi,
        gerekce,
    }
}

/// `konum` baytından itibaren `tetik` metinde geçiyorsa (bayt, sonlu) döner.
fn eslesme_baytlari(tetik: &Tetik, metin: &str, konum: usize) -> Option<(usize, usize)> {
    if konum >= metin.len() || !metin.is_char_boundary(konum) {
        return None;
    }
    // On eksiz tetik yalnizca kelime basinda eslesir; kayitparca icinde
    // parca tetigi tetiklenmez. On ekli tetikte ayracin kendisi
    // zaten sinirdir, bu yuzden o kosul aranmaz.
    if tetik.onek.is_none() && konum > 0 && !is_ayrac_bayt(metin.as_bytes()[konum - 1]) {
        return None;
    }
    let mut imlec = konum;

    if let Some(onek) = tetik.onek {
        let karakter = metin[konum..].chars().next()?;
        if karakter != onek {
            return None;
        }
        imlec = konum + karakter.len_utf8();
    }

    let mut bas = konum;
    for (sira, kelime) in tetik.kelimeler.iter().enumerate() {
        let (okunan, kelime_bas, bitis) = kelime_oku(metin, imlec)?;
        if &okunan != kelime {
            return None;
        }
        if sira > 0 && kelime_bas == imlec {
            // Araya ayraç girmeli; "abc" ile "ab" + "c" eşleşmesin.
            return None;
        }
        if sira == 0 && tetik.onek.is_none() {
            // On ekli tetikte eslesme on ekten baslar.
            bas = kelime_bas;
        }
        imlec = bitis;
    }
    Some((bas, imlec))
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn depo_kur(cesitler: &[(&str, u64)]) -> SablonDeposu {
        let mut depo = SablonDeposu::yeni();
        for (tetik, kullanim) in cesitler {
            let mut sablon = Sablon::yeni(tetik, "gövde");
            sablon.kullanim = *kullanim;
            depo.ekle(sablon).expect("eklenmeli");
        }
        depo
    }

    #[test]
    fn kucult_harf_turkce_i_harfini_dogru_indirgiyor() {
        assert_eq!(kucult_harf("IŞIK"), "işik");
        assert_eq!(kucult_harf("İstanbul"), "istanbul");
        assert_eq!(kucult_harf("ÇĞÖŞÜ"), "çğöşü");
    }

    #[test]
    fn kucult_harf_cok_baytli_harfleri_koruyor() {
        assert_eq!(kucult_harf("ÉCOLE"), "école");
        assert_eq!(kucult_harf("日本語"), "日本語");
    }

    #[test]
    fn tetik_ayiklama_onekli_ve_cokslu_kelimeli() {
        let t = Tetik::ayikla(";tesekkur ederim").expect("ayrıştırılmalı");
        assert_eq!(t.onek, Some(';'));
        assert_eq!(t.kelimeler, vec!["tesekkur", "ederim"]);
        assert_eq!(t.duz, ";tesekkur ederim");
        assert_eq!(t.kelime_sayisi(), 2);
    }

    #[test]
    fn tetik_ayiklama_oneksiz_kabul_ediliyor() {
        let t = Tetik::ayikla("is parcasi").expect("ayrıştırılmalı");
        assert_eq!(t.onek, None);
        assert_eq!(t.duz, "is parcasi");
    }

    #[test]
    fn tetik_ayiklama_bos_yazimi_reddediyor() {
        assert!(matches!(
            Tetik::ayikla("   "),
            Err(TypeFastHata::GecersizTetik { .. })
        ));
    }

    #[test]
    fn tetik_ayiklama_yalniz_ayrac_reddediliyor() {
        assert!(matches!(
            Tetik::ayikla(";;;"),
            Err(TypeFastHata::GecersizTetik { .. })
        ));
    }

    #[test]
    fn tetik_ayiklama_kelime_tasinin_reddediliyor() {
        let hata = Tetik::ayikla("bir iki uc dort bes").expect_err("hata bekleniyor");
        match hata {
            TypeFastHata::GecersizTetik { sebep, .. } => {
                assert!(sebep.contains("4"));
            }
            diger => panic!("beklenmeyen hata: {diger:?}"),
        }
    }

    #[test]
    fn tetik_ayiklama_ic_tireyi_kelimenin_ici_sayiyor() {
        let t = Tetik::ayikla("kayit-ekle").expect("ayrıştırılmalı");
        assert_eq!(t.kelimeler, vec!["kayit-ekle"]);
    }

    #[test]
    fn kelime_oku_ayraclari_atlayip_kelime_donduruyor() {
        let (kelime, bas, bit) = kelime_oku("Merhaba, dünya", 8).expect("kelime olmalı");
        assert_eq!(kelime, "dünya");
        assert_eq!(bas, 9);
        assert_eq!(bit, 15);
    }

    #[test]
    fn kelime_oku_son_disi_ayracli_kelimeyi_kirpiyor() {
        let (kelime, _, _) = kelime_oku("-kayit-", 0).expect("kelime olmalı");
        assert_eq!(kelime, "kayit");
    }

    #[test]
    fn kelime_oku_girdi_sonunda_none_donderiyor() {
        assert!(kelime_oku("abc", 3).is_none());
        assert!(kelime_oku("abc", 99).is_none());
    }

    #[test]
    fn cozumleyici_tetik_sayimini_dogru_sayiyor() {
        let depo = depo_kur(&[(";a", 0), ("b", 0), ("c d", 0)]);
        assert_eq!(Cozumleyici::yeni(&depo).uzunluk(), 3);
    }

    #[test]
    fn cozumleyici_bos_depoyla_hata_donderiyor() {
        let depo = SablonDeposu::yeni();
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert!(matches!(
            cozumleyici.coz("herhangi"),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn tam_eslesme_bulunuyor() {
        let depo = depo_kur(&[(";tesekkur", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz(";tesekkur").expect("cozulmeli");
        assert_eq!(cozum.tetik, ";tesekkur");
        assert_eq!(cozum.gerekce, CozumGerekcesi::Tek);
    }

    #[test]
    fn eslesme_ortadaki_noktalamayi_yutarak_bulunuyor() {
        let depo = depo_kur(&[("is parcasi", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici
            .coz("Merhaba, is parcasi olsun.")
            .expect("cozulmeli");
        assert_eq!(cozum.tetik, "is parcasi");
        assert_eq!(cozum.baslangic, 9);
        assert_eq!(cozum.tetik_yazimi, "is parcasi");
    }

    #[test]
    fn on_eksiz_tetik_oncarki_noktalamayi_yutuyor() {
        let depo = depo_kur(&[("tesekkur", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz(";tesekkur!").expect("cozulmeli");
        assert_eq!(cozum.tetik_yazimi, "tesekkur");
    }

    #[test]
    fn on_ek_gerekiyorsa_noktalamasiz_yazim_eslesmiyor() {
        let depo = depo_kur(&[(";tesekkur", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert!(matches!(
            cozumleyici.coz("tesekkur"),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn en_uzun_eslesme_kisayolu_yeniyor() {
        let mut depo = depo_kur(&[("is", 100), ("is parcasi", 0)]);
        depo.kullanim_artir("is").expect("sayac artmali");
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz("is parcasi").expect("cozulmeli");
        assert_eq!(cozum.tetik, "is parcasi");
    }

    #[test]
    fn tam_sayi_kurali_onekli_adayi_yeniyor() {
        let depo = depo_kur(&[(";is", 0), ("is", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz(";is").expect("cozulmeli");
        assert_eq!(cozum.tetik, ";is");
        assert_eq!(cozum.gerekce, CozumGerekcesi::TamSayi);
    }

    #[test]
    fn en_sik_kullanim_kurali_belirsizligi_cozuyor() {
        let depo = depo_kur(&[(";is", 9), ("is", 2)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz("Merhaba ;is").expect("cozulmeli");
        assert_eq!(cozum.tetik, ";is");
        assert_eq!(cozum.gerekce, CozumGerekcesi::EnSikKullanim);
    }

    #[test]
    fn esit_kullanimda_belirsizlik_hatasi_donderiliyor() {
        let depo = depo_kur(&[(";is", 3), ("is", 3)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        match cozumleyici.coz("Merhaba ;is") {
            Err(TypeFastHata::CozumBelirsiz { adaylar, .. }) => {
                assert_eq!(adaylar.len(), 2);
                assert!(adaylar.contains(&";is".to_string()));
                assert!(adaylar.contains(&"is".to_string()));
            }
            diger => panic!("belirsizlik bekleniyordu, gelen: {diger:?}"),
        }
    }

    #[test]
    fn kelime_bicimi_ayirac_olmadan_eslesmiyor() {
        let depo = depo_kur(&[("is parcasi", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert!(cozumleyici.coz("isparcasi").is_err());
    }

    #[test]
    fn kelime_ortasinda_eslesme_baslamiyor() {
        let depo = depo_kur(&[("parca", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert!(matches!(
            cozumleyici.coz("kayitparca"),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn turkce_tetik_unicode_karakterlerle_calisiyor() {
        let depo = depo_kur(&[(";şey", 0), ("görüşürüz", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz("Görüşürüz!").expect("cozulmeli");
        assert_eq!(cozum.tetik, "görüşürüz");
    }

    #[test]
    fn emoji_karisinda_eslesme_kaymadan_bulunuyor() {
        let depo = depo_kur(&[("teşekkür", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let cozum = cozumleyici.coz("🎉 teşekkür 🎉").expect("cozulmeli");
        assert_eq!(cozum.baslangic, 5);
    }

    #[test]
    fn tum_eslesmeler_cakismadan_diziliyor() {
        let depo = depo_kur(&[(";a", 0), (";b", 0), (";c", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let liste = cozumleyici.tum_eslesmeler(";a metin ;b ve ;c");
        let adlar: Vec<&str> = liste.iter().map(|c| c.tetik.as_str()).collect();
        assert_eq!(adlar, vec![";a", ";b", ";c"]);
        for pencere in liste.windows(2) {
            assert!(pencere[0].bitis <= pencere[1].baslangic);
        }
    }

    #[test]
    fn tum_eslesmeler_bos_girdide_bos_donduruyor() {
        let depo = depo_kur(&[(";a", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert!(cozumleyici.tum_eslesmeler("").is_empty());
    }

    #[test]
    fn tum_eslesmeler_belirsizlikte_durur() {
        let depo = depo_kur(&[(";is", 3), ("is", 3), (";son", 0)]);
        let cozumleyici = Cozumleyici::yeni(&depo);
        let liste = cozumleyici.tum_eslesmeler(";is ve ;son");
        assert!(liste.is_empty());
    }

    #[test]
    fn ilerleme_adimi_karakter_sinirini_koruyor() {
        let metin = "a🎉b";
        assert_eq!(ilerleme_adimi(metin, 0), 1);
        // "🎉" dört bayt: konum 1'den sonra 5 geçerli sınırdır.
        assert_eq!(ilerleme_adimi(metin, 1), 5);
        assert_eq!(ilerleme_adimi(metin, metin.len()), metin.len() + 1);
    }

    #[test]
    fn sozdizimsel_hatali_tetikler_cozumleyiciye_girmiyor() {
        let mut depo = SablonDeposu::yeni();
        depo.sablonlar.push(Sablon::yeni("!!!", "gövde"));
        let cozumleyici = Cozumleyici::yeni(&depo);
        assert_eq!(cozumleyici.uzunluk(), 0);
    }

    #[test]
    fn eslesme_baytlari_oneksiz_tetikte_basi_donduruyor() {
        let t = Tetik::ayikla("is").expect("ayrıştırılmalı");
        assert_eq!(eslesme_baytlari(&t, "Merhaba is", 8), Some((8, 10)));
        assert_eq!(eslesme_baytlari(&t, "Merhaba is", 0), None);
    }

    #[test]
    fn eslesme_baytlari_son_konumda_none_donderiyor() {
        let t = Tetik::ayikla("is").expect("ayrıştırılmalı");
        assert_eq!(eslesme_baytlari(&t, "is", 2), None);
    }
}
