//! Şablon varlığı ve düz metin (JSON) şablon deposu.
//!
//! Bir şablon beş bilgi taşır: **tetik** (kısayol), **gövde** (genişletilecek
//! metin), **kategori**, **açıklama** ve **kullanım sayacı**. Depo sınırlı bir
//! `Vec` içinde tutulur; her genişletmede sayac artar ve belirsizlik çözümünde
//! "en sık kullanım" ölçütü olarak devreye girer.
//!
//! Bu modül tetiğin **sözdizimsel doğrulamasını** yapar; metin içindeki eşleşme
//! ve çakışma çözümü [`crate::cozumle`] modülündedir.

use serde::{Deserialize, Serialize};

use crate::cozumle::{kucult_harf, Tetik};
use crate::hata::{Sonuc, TypeFastHata};

/// Depo belgesinin sürüm damgası.
///
/// Sürüm bilinmeyen ya da daha yeni bir belge reddedilir; eski sürüm okunur
/// ama uyarı üretmez (şema geriye dönük uyumludur).
pub const DEPO_SURUMU: u32 = 1;

/// Tek bir genişletilebilir şablon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sablon {
    /// Kısayol tetiği; nokta, boşluk veya noktalama ile ayrılmış kelime dizisi.
    pub tetik: String,
    /// Genişletilecek ham metin.
    pub govde: String,
    /// Kullanıcı tanımlı kategori (ör. `iletisim`, `rapor`).
    #[serde(default)]
    pub kategori: String,
    /// Kullanıcıya hatırlatma yapan kısa açıklama.
    #[serde(default)]
    pub aciklama: String,
    /// Gizli mod işareti: çıktı maskelenir ve pano geçmişine yazılmaz.
    #[serde(default)]
    pub gizli: bool,
    /// Kaç kez genişletildiğini sayan sayaç.
    #[serde(default)]
    pub kullanim: u64,
}

impl Sablon {
    /// Yalnızca tetik ve gövde doldurulmuş yeni bir şablon üretir.
    pub fn yeni(tetik: &str, govde: &str) -> Self {
        Self {
            tetik: tetik.to_string(),
            govde: govde.to_string(),
            kategori: String::new(),
            aciklama: String::new(),
            gizli: false,
            kullanim: 0,
        }
    }

    /// Tetiğin söz dizimsel biçimini doğrular ve normalleştirilmiş hâlini döner.
    ///
    /// # Hatalar
    ///
    /// Tetik boşsa, ayraçları geçersizse, kelime sayısı sınırı aşılırsa veya
    /// gövde boşsa [`TypeFastHata::GecersizTetik`] döner.
    pub fn tetigi_cozumle(&self) -> Sonuc<Tetik> {
        if self.govde.trim().is_empty() {
            return Err(TypeFastHata::GecersizTetik {
                tetik: self.tetik.clone(),
                sebep: "genişletilecek gövde boş olamaz".to_string(),
            });
        }
        Tetik::ayikla(&self.tetik)
    }

    /// Tetiğin büyük/küçük harf ve gereksiz boşluklardan arındırılmış yazımı.
    pub fn tetik_anahtari(&self) -> String {
        self.tetik.trim().to_lowercase().replace(' ', "")
    }

    /// `add` çizgisinde gösterilecek kısa tanım.
    pub fn ozet(&self) -> String {
        let govde_ozet: String = self.govde.chars().take(48).collect();
        if self.govde.chars().count() > 48 {
            format!("{govde_ozet}…")
        } else {
            govde_ozet
        }
    }
}

/// Şablon koleksiyonu; depo dosyasının bellekteki karşılığı.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SablonDeposu {
    /// Belge sürüm damgası.
    #[serde(default = "varsayilan_surum")]
    pub surum: u32,
    /// Şablon listesi; tetiğe göre kararlı sırada tutulur.
    #[serde(default)]
    pub sablonlar: Vec<Sablon>,
}

fn varsayilan_surum() -> u32 {
    DEPO_SURUMU
}

impl SablonDeposu {
    /// Boş depo üretir.
    pub fn yeni() -> Self {
        Self {
            surum: DEPO_SURUMU,
            sablonlar: Vec::new(),
        }
    }

    /// Depodaki şablon sayısı.
    pub fn uzunluk(&self) -> usize {
        self.sablonlar.len()
    }

    /// Depo boş mu?
    pub fn bos(&self) -> bool {
        self.sablonlar.is_empty()
    }

    /// Yeni şablon ekler.
    ///
    /// Tetik çakışması **burada** reddedilir: sessizce üzerine yazma yapılmaz,
    /// çünkü iki farklı kısayol aynı metni genişletirse kullanıcı hangisinin
    /// kullandığını bilemez (rapor `b03` — S5).
    ///
    /// # Hatalar
    ///
    /// Tetik geçersizse veya zaten varsa hata döner.
    pub fn ekle(&mut self, sablon: Sablon) -> Sonuc<()> {
        let tetik = sablon.tetigi_cozumle()?;
        if let Some(var_olan) = self.bul(&tetik.duz) {
            return Err(TypeFastHata::TetikCakismasi {
                tetik: sablon.tetik.clone(),
                mevcut: var_olan.ozet(),
            });
        }
        let anahtar = sablon.tetik_anahtari();
        let konum = self
            .sablonlar
            .iter()
            .position(|s| s.tetik_anahtari() > anahtar)
            .unwrap_or(self.sablonlar.len());
        self.sablonlar.insert(konum, sablon);
        Ok(())
    }

    /// Normalleştirilmiş tetiğe göre ilk eşleşen şablonu bulur.
    pub fn bul(&self, tetik: &str) -> Option<&Sablon> {
        let anahtar = normalize_anahtar(tetik);
        self.sablonlar
            .iter()
            .find(|s| s.tetik_anahtari() == anahtar)
    }

    /// Normalleştirilmiş tetiğe göle şablonun dizinini döner.
    pub fn konum(&self, tetik: &str) -> Option<usize> {
        let anahtar = normalize_anahtar(tetik);
        self.sablonlar
            .iter()
            .position(|s| s.tetik_anahtari() == anahtar)
    }

    /// Tetiği siler; yoksa `false` döner.
    pub fn sil(&mut self, tetik: &str) -> bool {
        match self.konum(tetik) {
            Some(konum) => {
                self.sablonlar.remove(konum);
                true
            }
            None => false,
        }
    }

    /// Tetiğin kullanım sayacını bir artırır.
    ///
    /// # Hatalar
    ///
    /// Tetik depoda yoksa [`TypeFastHata::BilinmeyenSablon`] döner.
    pub fn kullanim_artir(&mut self, tetik: &str) -> Sonuc<()> {
        match self.konum(tetik) {
            Some(konum) => {
                self.sablonlar[konum].kullanim = self.sablonlar[konum].kullanim.saturating_add(1);
                Ok(())
            }
            None => Err(TypeFastHata::BilinmeyenSablon {
                tetik: tetik.to_string(),
            }),
        }
    }

    /// Tüm şablonların değişmez sıralı kopyası.
    pub fn tumune(&self) -> &[Sablon] {
        &self.sablonlar
    }

    /// `kategori` filtresiyle döner; `kategori` boşsa tümünü verir.
    pub fn kategoriye_gore(&self, kategori: &str) -> Vec<&Sablon> {
        if kategori.is_empty() {
            return self.sablonlar.iter().collect();
        }
        let hedef = kucult_harf(kategori);
        self.sablonlar
            .iter()
            .filter(|s| kucult_harf(&s.kategori) == hedef)
            .collect()
    }

    /// Depodaki tüm kategori adları (tekrarsız, alfabetik).
    pub fn kategoriler(&self) -> Vec<String> {
        let mut liste: Vec<String> = self
            .sablonlar
            .iter()
            .map(|s| s.kategori.clone())
            .filter(|k| !k.is_empty())
            .collect();
        liste.sort_by_key(|ad| kucult_harf(ad));
        liste.dedup();
        liste
    }

    /// En çok kullanılan ilk `adet` şablon.
    ///
    /// Kullanım sayacı eşitse tetiğin alfabetik sırası karar verir; çıktı
    /// her koşulda kararlıdır.
    pub fn en_cok_kullanilan(&self, adet: usize) -> Vec<&Sablon> {
        let mut sirali: Vec<&Sablon> = self.sablonlar.iter().collect();
        sirali.sort_by(|a, b| {
            b.kullanim
                .cmp(&a.kullanim)
                .then_with(|| a.tetik.cmp(&b.tetik))
        });
        sirali.truncate(adet);
        sirali
    }
}

/// Tetiği karşılaştırma anahtarına çevirir: kısa harfe indirger, boşlukları siler.
fn normalize_anahtar(tetik: &str) -> String {
    tetik.split_whitespace().map(kucult_harf).collect()
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn ornek_depo() -> SablonDeposu {
        let mut depo = SablonDeposu::yeni();
        let a = Sablon::yeni(";tesekkur", "Merhaba, teşekkür ederim.");
        let b = Sablon::yeni("is parcasi", "Merhaba, nasıl yardımcı olabilirim?");
        depo.ekle(a).expect("a eklenmeli");
        depo.ekle(b).expect("b eklenmeli");
        depo
    }

    #[test]
    fn yeni_sablon_alanlari_dogru_dolduruluyor() {
        let s = Sablon::yeni(";x", "y");
        assert_eq!(s.tetik, ";x");
        assert_eq!(s.govde, "y");
        assert!(s.kategori.is_empty());
        assert!(!s.gizli);
        assert_eq!(s.kullanim, 0);
    }

    #[test]
    fn bos_govde_tetik_denetimi_reddediyor() {
        let s = Sablon::yeni(";x", "   ");
        let hata = s.tetigi_cozumle().expect_err("hata bekleniyor");
        assert!(matches!(hata, TypeFastHata::GecersizTetik { .. }));
    }

    #[test]
    fn bos_tetik_reddediliyor() {
        let s = Sablon::yeni("   ", "gövde");
        assert!(matches!(
            s.tetigi_cozumle(),
            Err(TypeFastHata::GecersizTetik { .. })
        ));
    }

    #[test]
    fn ozet_uzun_govdede_kisaltma_isareti_koyuyor() {
        let uzun = "x".repeat(80);
        let s = Sablon::yeni(";x", &uzun);
        assert!(s.ozet().ends_with('…'));
        let kisa = Sablon::yeni(";x", "kısa");
        assert_eq!(kisa.ozet(), "kısa");
    }

    #[test]
    fn depo_bos_durumda_bos_isaretliyor() {
        let depo = SablonDeposu::yeni();
        assert!(depo.bos());
        assert_eq!(depo.uzunluk(), 0);
    }

    #[test]
    fn ekleme_listeyi_alfabetik_siraliyor() {
        let mut depo = SablonDeposu::yeni();
        for ad in ["zeta", "alfa", "mu"] {
            depo.ekle(Sablon::yeni(ad, "gövde")).expect("eklenmeli");
        }
        let sirali: Vec<&str> = depo.tumune().iter().map(|s| s.tetik.as_str()).collect();
        assert_eq!(sirali, vec!["alfa", "mu", "zeta"]);
    }

    #[test]
    fn ayni_tetik_iki_kez_eklenemiyor() {
        let mut depo = ornek_depo();
        let hata = depo
            .ekle(Sablon::yeni(";tesekkur", "başka gövde"))
            .expect_err("çakışma bekleniyor");
        match hata {
            TypeFastHata::TetikCakismasi { tetik, mevcut } => {
                assert_eq!(tetik, ";tesekkur");
                assert!(!mevcut.is_empty());
            }
            diger => panic!("beklenmeyen hata: {diger:?}"),
        }
    }

    #[test]
    fn buyuk_kucuk_harf_farkli_tetik_sayiliyor() {
        let mut depo = ornek_depo();
        assert!(depo.ekle(Sablon::yeni(";Teşekkür", "başka")).is_ok());
        assert_eq!(depo.uzunluk(), 3);
    }

    #[test]
    fn bul_ve_konum_birbirini_tutarli() {
        let depo = ornek_depo();
        assert!(depo.bul(";tesekkur").is_some());
        let konum = depo.konum(";tesekkur").expect("konum olmalı");
        assert_eq!(depo.tumune()[konum].tetik, ";tesekkur");
        assert!(depo.bul("olmayan").is_none());
        assert!(depo.konum("olmayan").is_none());
    }

    #[test]
    fn silme_var_olmayan_tetikte_false_donduruyor() {
        let mut depo = ornek_depo();
        assert!(depo.sil(";tesekkur"));
        assert!(!depo.sil(";tesekkur"));
        assert_eq!(depo.uzunluk(), 1);
    }

    #[test]
    fn kullanim_sayaci_artiyor() {
        let mut depo = ornek_depo();
        depo.kullanim_artir(";tesekkur").expect("artmalı");
        depo.kullanim_artir(";tesekkur").expect("artmalı");
        assert_eq!(depo.bul(";tesekkur").map(|s| s.kullanim), Some(2));
    }

    #[test]
    fn kullanim_olmayan_tetikte_hata_donderiyor() {
        let mut depo = ornek_depo();
        assert!(matches!(
            depo.kullanim_artir("yok"),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn kategoriye_gore_filtreliyor() {
        let mut depo = SablonDeposu::yeni();
        let mut a = Sablon::yeni(";tesekkur", "gövde");
        a.kategori = "İletişim".to_string();
        let mut b = Sablon::yeni("is parcasi", "gövde");
        b.kategori = "rapor".to_string();
        depo.ekle(a).expect("a eklenmeli");
        depo.ekle(b).expect("b eklenmeli");

        let eslesen = depo.kategoriye_gore("İLETİŞİM");
        assert_eq!(eslesen.len(), 1);
        assert_eq!(eslesen[0].tetik, ";tesekkur");
        assert_eq!(depo.kategoriye_gore("").len(), 2);
    }

    #[test]
    fn kategorisi_olmayan_sablon_tum_sorgulere_dahil() {
        let depo = ornek_depo();
        assert_eq!(depo.tumune().len(), 2);
    }

    #[test]
    fn kategoriler_tekrarsiz_ve_sirali() {
        let mut depo = SablonDeposu::yeni();
        for (ad, kat) in [("a", "rapor"), ("b", "iletisim"), ("c", "rapor"), ("d", "")] {
            let mut s = Sablon::yeni(ad, "gövde");
            s.kategori = kat.to_string();
            depo.ekle(s).expect("eklenmeli");
        }
        assert_eq!(depo.kategoriler(), vec!["iletisim", "rapor"]);
    }

    #[test]
    fn en_cok_kullanilan_sayaci_ve_tetigi_siraliyor() {
        let mut depo = ornek_depo();
        depo.kullanim_artir(";tesekkur").expect("artmalı");
        depo.kullanim_artir("is parcasi").expect("artmalı");
        depo.kullanim_artir("is parcasi").expect("artmalı");
        let liste = depo.en_cok_kullanilan(2);
        assert_eq!(liste[0].tetik, "is parcasi");
        assert_eq!(liste[0].kullanim, 2);
        assert_eq!(liste[1].tetik, ";tesekkur");
    }

    #[test]
    fn en_cok_kullanilan_ada_tutarsinca_alfa_artik() {
        let depo = ornek_depo();
        let liste = depo.en_cok_kullanilan(2);
        assert_eq!(liste[0].tetik, ";tesekkur");
    }

    #[test]
    fn en_cok_kullanilan_liste_kisaltiliyor() {
        let depo = ornek_depo();
        assert_eq!(depo.en_cok_kullanilan(1).len(), 1);
        assert!(depo.en_cok_kullanilan(0).is_empty());
    }

    #[test]
    fn tetik_anahtari_bosluklari_sildiyor() {
        let s = Sablon::yeni("  Is   Parcasi ", "gövde");
        assert_eq!(s.tetik_anahtari(), "isparcasi");
    }

    #[test]
    fn bos_alanlarla_json_gidis_donusu_calisiyor() {
        let s = Sablon::yeni(";x", "gövde");
        let metin = serde_json::to_string(&s).expect("serileştirilmeli");
        let geri: Sablon = serde_json::from_str(&metin).expect("ayrıştırılmalı");
        assert_eq!(s, geri);
    }

    #[test]
    fn eksik_alanlar_varsayilan_degerle_dolduruluyor() {
        let geri: Sablon =
            serde_json::from_str(r#"{"tetik":";x","govde":"gövde"}"#).expect("ayrıştırılmalı");
        assert!(!geri.gizli);
        assert_eq!(geri.kullanim, 0);
        assert!(geri.kategori.is_empty());
    }

    #[test]
    fn depo_belgesi_json_gidis_donusu_calisiyor() {
        let depo = ornek_depo();
        let metin = serde_json::to_string_pretty(&depo).expect("serileştirilmeli");
        let geri: SablonDeposu = serde_json::from_str(&metin).expect("ayrıştırılmalı");
        assert_eq!(depo, geri);
    }
}
