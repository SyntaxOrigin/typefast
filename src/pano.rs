//! Sınırlı pano geçmişi — halka tampon.
//!
//! Kopyalanan her şey kaydedilmez. Kayıt sayısı, tek öğe boyutu ve saklama
//! sırası sınırlıdır; sınır aşıldığında **sessizce taşma olmaz**, "atlandı"
//! sayacı artar (rapor `b03` — S3). Diskteki dosya **düz metindir**; şifreleme
//! kapsam dışıdır ve README'nin en üstündeki uyarıda yazılıdır.
//!
//! Yapı `VecDeque` üzerine kuruludur. `VecDeque` halka tampon olduğu için
//! baştan silme maliyeti `O(1)`'dir; kapasite aşımında en eski kayıt sondan
//! düşer.

use serde::{Deserialize, Serialize};

use crate::cozumle::kucult_harf;
use crate::gizli;

/// Varsayılan kayıt sınırı.
pub const VARSAYILAN_KAPASITE: usize = 200;

/// Varsayılan tek öğe boyut sınırı (karakter sayısı).
pub const VARSAYILAN_MAKS_BOYUT: usize = 4096;

/// Panoya eklenen tek bir kayıt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanoKaydi {
    /// Artan sıra numarası; kayıt silinse bile yeniden kullanılmaz.
    pub id: u64,
    /// Kaydın özgün metni (düz metin saklanır).
    pub metin: String,
    /// Gizli mod işareti: kalıp denetimi sır buldu.
    #[serde(default)]
    pub gizli: bool,
    /// Kaydedildiği an (Unix saniye; istemci tarafından verilir).
    #[serde(default)]
    pub zaman: u64,
    /// Metnin karakter sayısı.
    #[serde(default)]
    pub boyut: usize,
}

impl PanoKaydi {
    /// Kaydın gösterilecek hâlini üretir.
    ///
    /// Gizli kayıtlar **kendiliğinden** maskelenir; özgün metin ancak
    /// `coz` isteğiyle geri döner (rapor `b03` — S4).
    pub fn gorunen_metin(&self) -> String {
        if self.gizli {
            gizli::tumunu_maskele(&self.metin)
        } else {
            gizli::maskele(&self.metin).metin
        }
    }

    /// Kaydın arama için normalize edilmiş metni.
    fn arama_metni(&self) -> String {
        kucult_harf(&self.metin)
    }
}

/// Bir kaydın kabul edilip edilmediğini bildiren sonuç.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanoKabul {
    /// Yeni kayıt eklendi; verilen numara kaydın kimliğidir.
    Eklendi(u64),
    /// Aynı metin zaten vardı; kayıt en başa taşındı.
    Tekrar(u64),
    /// Boş metin veya boyut sınırı aşıldı; kaydedilmedi.
    Atlandi,
}

/// Pano geçmişi belgesi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanoGecmisi {
    /// Belge sürüm damgası.
    #[serde(default = "varsayilan_surum")]
    pub surum: u32,
    /// Saklanacak en fazla kayıt sayısı.
    #[serde(default = "varsayilan_kapasite")]
    pub kapasite: usize,
    /// Tek öğe için izin verilen en fazla karakter sayısı.
    #[serde(default = "varsayilan_maks_boyut")]
    pub maks_boyut: usize,
    /// En eskiden en yeniye kayıtlar.
    #[serde(default)]
    pub kayitlar: Vec<PanoKaydi>,
    /// Bir sonraki kayıt numarası.
    #[serde(default = "varsayilan_sonraki_id")]
    pub sonraki_id: u64,
    /// Boyut sınırı veya boşluk nedeniyle atlanan kayıt sayısı.
    #[serde(default)]
    pub atlanan: u64,
}

fn varsayilan_surum() -> u32 {
    crate::sablon::DEPO_SURUMU
}

fn varsayilan_kapasite() -> usize {
    VARSAYILAN_KAPASITE
}

fn varsayilan_maks_boyut() -> usize {
    VARSAYILAN_MAKS_BOYUT
}

fn varsayilan_sonraki_id() -> u64 {
    1
}

impl Default for PanoGecmisi {
    fn default() -> Self {
        Self::yeni(VARSAYILAN_KAPASITE, VARSAYILAN_MAKS_BOYUT)
    }
}

impl PanoGecmisi {
    /// Verilen sınırlarla boş geçmiş üretir.
    ///
    /// `kapasite` ve `maks_boyut` sıfırsa aracın varsayılanı kullanılır; sınır
    /// kaldırılmaz.
    pub fn yeni(kapasite: usize, maks_boyut: usize) -> Self {
        Self {
            surum: varsayilan_surum(),
            kapasite: if kapasite == 0 {
                VARSAYILAN_KAPASITE
            } else {
                kapasite
            },
            maks_boyut: if maks_boyut == 0 {
                VARSAYILAN_MAKS_BOYUT
            } else {
                maks_boyut
            },
            kayitlar: Vec::new(),
            sonraki_id: 1,
            atlanan: 0,
        }
    }

    /// Kayıt sayısı.
    pub fn uzunluk(&self) -> usize {
        self.kayitlar.len()
    }

    /// Geçmiş boş mu?
    pub fn bos(&self) -> bool {
        self.kayitlar.is_empty()
    }

    /// Metni geçmişe ekler.
    ///
    /// Boş metin ve boyut sınırını aşan metin kaydedilmez; `atlanan` sayacı
    /// artar. Aynı metin ikinci kez eklenirse yeni kayıt açılmaz, mevcut
    /// kayıt en başa taşınır.
    pub fn ekle(&mut self, metin: &str, zaman: u64) -> PanoKabul {
        let boyut = metin.chars().count();
        if metin.is_empty() || boyut > self.maks_boyut {
            self.atlanan = self.atlanan.saturating_add(1);
            return PanoKabul::Atlandi;
        }
        if let Some(konum) = self.kayitlar.iter().position(|k| k.metin == metin) {
            let kayit = self.kayitlar.remove(konum);
            self.kayitlar.insert(0, kayit);
            return PanoKabul::Tekrar(self.kayitlar[0].id);
        }

        let kayit = PanoKaydi {
            id: self.sonraki_id,
            metin: metin.to_string(),
            gizli: !gizli::bul(metin).is_empty(),
            zaman,
            boyut,
        };
        self.sonraki_id = self.sonraki_id.saturating_add(1);
        self.kayitlar.insert(0, kayit);
        while self.kayitlar.len() > self.kapasite {
            self.kayitlar.pop();
        }
        PanoKabul::Eklendi(self.kayitlar[0].id)
    }

    /// En yeni kayıttan eskiye doğru listeler.
    pub fn listele(&self) -> &[PanoKaydi] {
        &self.kayitlar
    }

    /// Kayıt numarasına göre arar.
    pub fn id_ile(&self, id: u64) -> Option<&PanoKaydi> {
        self.kayitlar.iter().find(|k| k.id == id)
    }

    /// Alt dizi araması yapar (büyük/küçük harf duyarsız).
    ///
    /// `gizliler_dahil` `false` iken gizli işaretli kayıtlar sonuçta yer almaz;
    /// arama sonucunda da maskeli gösterilir.
    pub fn ara(&self, sorgu: &str, gizliler_dahil: bool) -> Vec<&PanoKaydi> {
        if sorgu.is_empty() {
            return Vec::new();
        }
        let hedef = kucult_harf(sorgu);
        self.kayitlar
            .iter()
            .filter(|k| gizliler_dahil || !k.gizli)
            .filter(|k| k.arama_metni().contains(&hedef))
            .collect()
    }

    /// Sır kalıbı içeren kayıtları numaralarıyla döner.
    pub fn gizli_kayitlar(&self) -> Vec<&PanoKaydi> {
        self.kayitlar.iter().filter(|k| k.gizli).collect()
    }

    /// Geçmişi siler; sayaçlar sıfırlanmaz, "atlanan" korunur.
    pub fn temizle(&mut self) {
        self.kayitlar.clear();
    }

    /// Kayıt numarasına göre tek kaydı siler.
    pub fn sil(&mut self, id: u64) -> bool {
        let onceki = self.kayitlar.len();
        self.kayitlar.retain(|k| k.id != id);
        self.kayitlar.len() != onceki
    }
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn gecmis() -> PanoGecmisi {
        PanoGecmisi::yeni(3, 10)
    }

    fn genis_gecmis() -> PanoGecmisi {
        PanoGecmisi::yeni(10, 200)
    }

    #[test]
    fn bos_gecmis_ozellikleri_dogru() {
        let g = gecmis();
        assert!(g.bos());
        assert_eq!(g.uzunluk(), 0);
        assert_eq!(g.sonraki_id, 1);
        assert_eq!(g.atlanan, 0);
    }

    #[test]
    fn sifir_kapasite_varsayilana_duser() {
        let g = PanoGecmisi::yeni(0, 0);
        assert_eq!(g.kapasite, VARSAYILAN_KAPASITE);
        assert_eq!(g.maks_boyut, VARSAYILAN_MAKS_BOYUT);
    }

    #[test]
    fn varsayilan_gecmisi_degerleri_gecerli() {
        let g = PanoGecmisi::default();
        assert_eq!(g.kapasite, VARSAYILAN_KAPASITE);
        assert!(g.bos());
    }

    #[test]
    fn ekleme_kaydi_en_basa_koyuyor() {
        let mut g = gecmis();
        g.ekle("bir", 1);
        g.ekle("iki", 2);
        assert_eq!(g.listele()[0].metin, "iki");
        assert_eq!(g.listele()[1].metin, "bir");
    }

    #[test]
    fn ayni_metin_tekrar_eklenirse_tasiniyor() {
        let mut g = gecmis();
        g.ekle("bir", 1);
        g.ekle("iki", 2);
        match g.ekle("bir", 3) {
            PanoKabul::Tekrar(id) => {
                assert_eq!(id, 1);
                assert_eq!(g.uzunluk(), 2);
                assert_eq!(g.listele()[0].metin, "bir");
            }
            diger => panic!("beklenmeyen kabul: {diger:?}"),
        }
    }

    #[test]
    fn bos_metin_atlandi_sayacini_artiriyor() {
        let mut g = gecmis();
        assert_eq!(g.ekle("", 1), PanoKabul::Atlandi);
        assert_eq!(g.atlanan, 1);
        assert!(g.bos());
    }

    #[test]
    fn boyut_siniri_asan_metin_atlandi_sayacini_artiriyor() {
        let mut g = gecmis();
        let uzun = "x".repeat(11);
        assert_eq!(g.ekle(&uzun, 1), PanoKabul::Atlandi);
        assert_eq!(g.atlanan, 1);
        assert!(g.bos());
    }

    #[test]
    fn boyut_sinirindaki_metin_kabul_ediliyor() {
        let mut g = gecmis();
        let tam = "x".repeat(10);
        assert!(matches!(g.ekle(&tam, 1), PanoKabul::Eklendi(_)));
    }

    #[test]
    fn halka_tampon_kapasiteyi_asmaz() {
        let mut g = gecmis();
        for i in 0..6 {
            g.ekle(&format!("m{i}"), i);
        }
        assert_eq!(g.uzunluk(), 3);
        assert_eq!(g.listele()[0].metin, "m5");
        assert_eq!(g.listele()[2].metin, "m3");
    }

    #[test]
    fn gizli_kalip_olan_kayit_isaretleniyor() {
        let mut g = genis_gecmis();
        g.ekle("parola: gizli123", 1);
        assert!(g.listele()[0].gizli);
        assert_eq!(g.gizli_kayitlar().len(), 1);
    }

    #[test]
    fn duz_kayit_isaretlenmiyor() {
        let mut g = gecmis();
        g.ekle("merhaba", 1);
        assert!(!g.listele()[0].gizli);
        assert!(g.gizli_kayitlar().is_empty());
    }

    #[test]
    fn gizli_kayit_goruntulenirken_maskeleniyor() {
        let mut g = genis_gecmis();
        g.ekle("parola: gizli123", 1);
        let gorunen = g.listele()[0].gorunen_metin();
        assert!(!gorunen.contains("gizli123"));
        assert!(gorunen.starts_with(gizli::MASKE_ISARETI));
    }

    #[test]
    fn gizli_kaydin_ozgun_metni_geri_donusu_mumkun() {
        let mut g = genis_gecmis();
        g.ekle("parola: gizli123", 1);
        let kayit = g.id_ile(1).expect("kayit olmali");
        assert_eq!(kayit.metin, "parola: gizli123");
    }

    #[test]
    fn arama_gizlileri_varsayilan_dahil_etmiyor() {
        let mut g = genis_gecmis();
        g.ekle("parola: gizli123", 1);
        g.ekle("rapor metni", 2);
        assert_eq!(g.ara("gizli123", false).len(), 0);
        assert_eq!(g.ara("gizli123", true).len(), 1);
    }

    #[test]
    fn arama_harf_duyarsiz_calisiyor() {
        let mut g = gecmis();
        g.ekle("Fatura No", 1);
        assert_eq!(g.ara("fatura", false).len(), 1);
        assert_eq!(g.ara("FATURA", false).len(), 1);
    }

    #[test]
    fn bos_sorgu_sonuc_dondurmuyor() {
        let mut g = gecmis();
        g.ekle("metin", 1);
        assert!(g.ara("", false).is_empty());
    }

    #[test]
    fn id_ile_bulma_calisiyor() {
        let mut g = gecmis();
        g.ekle("bir", 1);
        assert_eq!(g.id_ile(1).map(|k| k.metin.as_str()), Some("bir"));
        assert!(g.id_ile(99).is_none());
    }

    #[test]
    fn temizleme_kayitlari_siliyor_sayaclari_koruyor() {
        let mut g = gecmis();
        g.ekle("bir", 1);
        g.ekle("", 2);
        g.temizle();
        assert!(g.bos());
        assert_eq!(g.atlanan, 1);
        assert_eq!(g.sonraki_id, 2);
    }

    #[test]
    fn tek_kayit_silme_calisiyor() {
        let mut g = gecmis();
        g.ekle("bir", 1);
        g.ekle("iki", 2);
        assert!(g.sil(1));
        assert!(!g.sil(1));
        assert_eq!(g.uzunluk(), 1);
    }

    #[test]
    fn kayit_boyutu_karakter_sayiliyor() {
        let mut g = gecmis();
        g.ekle("şğü", 1);
        assert_eq!(g.listele()[0].boyut, 3);
    }

    #[test]
    fn gecmis_json_gidis_donusu_calisiyor() {
        let mut g = gecmis();
        g.ekle("parola: gizli123", 5);
        g.ekle("rapor", 6);
        let metin = serde_json::to_string(&g).expect("serilestirilmeli");
        let geri: PanoGecmisi = serde_json::from_str(&metin).expect("ayristirilmeli");
        assert_eq!(g, geri);
    }

    #[test]
    fn eksik_alanlarla_json_ayristiriliyor() {
        let geri: PanoGecmisi = serde_json::from_str("{}").expect("ayristirilmeli");
        assert_eq!(geri.kapasite, VARSAYILAN_KAPASITE);
        assert_eq!(geri.sonraki_id, 1);
        assert!(geri.bos());
    }
}
