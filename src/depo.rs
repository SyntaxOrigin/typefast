//! Depo dosyalarının yüklenmesi, kaydedilmesi ve **atomik yazma**.
//!
//! Üç ayrı düz metin dosya kullanılır ve bu ayrım bilinçlidir:
//!
//! | Dosya | İçerik | Neden ayrı |
//! |---|---|---|
//! | `sablonlar.json` | Şablon koleksiyonu | Kısayol koleksiyonu yedeklenir. |
//! | `pano.json` | Pano geçmişi | Kullanıcı geçmişi tek başına silebilir. |
//! | `istatistik.json` | Kazanç sayaçları | Silmek çalışmayı bozmaz. |
//!
//! Her yazma **atomiktir**: önce aynı dizinde `*.tmp` geçici dosyasına yazılır,
//! sonra `fs::rename` ile hedefe taşınır. Böylece yazma sırasında bir hata
//! olursa eski dosya bozulmadan kalır.
//!
//! **Şifreleme yoktur.** `README.md` → `## Bilinen Sınırlamalar` ve dosya
//! başındaki uyarı bunu açıkça yazar.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::hata::{okuma_hatasi, yazma_hatasi, Sonuc, TypeFastHata};
use crate::kazanc::Istatistik;
use crate::pano::PanoGecmisi;
use crate::sablon::SablonDeposu;

/// Depo dosyalarının bulunduğu dizini temsil eder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepoYolu {
    /// Dosyaların bulunacağı dizin.
    pub kok: PathBuf,
}

impl DepoYolu {
    /// Verilen dizinden bir depo yolu üretir.
    pub fn yeni(kok: impl Into<PathBuf>) -> Self {
        Self { kok: kok.into() }
    }

    /// Şablon dosyasının tam yolu.
    pub fn sablon_dosyasi(&self) -> PathBuf {
        self.kok.join("sablonlar.json")
    }

    /// Pano geçmişi dosyasının tam yolu.
    pub fn pano_dosyasi(&self) -> PathBuf {
        self.kok.join("pano.json")
    }

    /// İstatistik dosyasının tam yolu.
    pub fn istatistik_dosyasi(&self) -> PathBuf {
        self.kok.join("istatistik.json")
    }
}

/// Bir `&str` yolundan depo yolu üretir.
pub fn yoldan(yol: &str) -> DepoYolu {
    let yol = Path::new(yol);
    if yol.extension().is_some() {
        DepoYolu::yeni(yol.parent().unwrap_or_else(|| Path::new(".")))
    } else {
        DepoYolu::yeni(yol)
    }
}

/// Şablon deposunu yükler; dosya yoksa boş depo döner.
pub fn sablonlari_yukle(yol: &DepoYolu) -> Sonuc<SablonDeposu> {
    yukle_or_bos(&yol.sablon_dosyasi())
}

/// Pano geçmişini yükler; dosya yoksa boş geçmiş döner.
pub fn panoyu_yukle(yol: &DepoYolu) -> Sonuc<PanoGecmisi> {
    yukle_or_bos(&yol.pano_dosyasi())
}

/// İstatistiği yükler; dosya yoksa boş (ölçülmemiş) istatistik döner.
pub fn istatistigi_yukle(yol: &DepoYolu) -> Sonuc<Istatistik> {
    yukle_or_bos(&yol.istatistik_dosyasi())
}

/// Şablon deposunu atomik olarak yazar.
pub fn sablonlari_kaydet(yol: &DepoYolu, depo: &SablonDeposu) -> Sonuc<()> {
    atomik_yaz(&yol.sablon_dosyasi(), depo)
}

/// Pano geçmişini atomik olarak yazar.
pub fn panoyu_kaydet(yol: &DepoYolu, pano: &PanoGecmisi) -> Sonuc<()> {
    atomik_yaz(&yol.pano_dosyasi(), pano)
}

/// İstatistiği atomik olarak yazar.
pub fn istatistigi_kaydet(yol: &DepoYolu, istatistik: &Istatistik) -> Sonuc<()> {
    atomik_yaz(&yol.istatistik_dosyasi(), istatistik)
}

/// Bayt dizisini ayni mantikla atomik olarak diske yazar.
///
/// Disa/içe aktarma belgeleri JSON olmadigi icin `atomik_yaz` yerine bu
/// ham bayt surumu kullanilir; gecici dosya + rename adimi aynidir.
pub fn atomik_yaz_dosya(dosya: &Path, baytlar: &[u8]) -> Sonuc<()> {
    if let Some(ust) = dosya.parent() {
        if !ust.as_os_str().is_empty() {
            fs::create_dir_all(ust).map_err(|hata| yazma_hatasi(ust, hata))?;
        }
    }
    let gecici = gecici_yol(dosya);
    fs::write(&gecici, baytlar).map_err(|hata| yazma_hatasi(&gecici, hata))?;
    if let Err(hata) = fs::rename(&gecici, dosya) {
        let _ = fs::remove_file(&gecici);
        return Err(yazma_hatasi(dosya, hata));
    }
    Ok(())
}

/// JSON'u okur; dosya yoksa `varsayilan` degerini doner.
pub fn yukle_or_bos<T: DeserializeOwned + Default>(dosya: &Path) -> Sonuc<T> {
    if !dosya.exists() {
        return Ok(T::default());
    }
    let metin = fs::read_to_string(dosya).map_err(|hata| okuma_hatasi(dosya, hata))?;
    if metin.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str(&metin).map_err(|hata| TypeFastHata::BozukDepo {
        yol: dosya.to_path_buf(),
        ayrinti: hata.to_string(),
    })
}

/// JSON'u iki boş satırla atomik olarak diske yazar.
///
/// `kok` dizini yoksa oluşturulur. Yazma sırasında oluşan geçici dosya
/// adlandırması `.tmp` ile biter; yarım kalmış bir yazma hedef dosyaya
/// **yansımaz**.
pub fn atomik_yaz<T: Serialize>(dosya: &Path, icerik: &T) -> Sonuc<()> {
    if let Some(ust) = dosya.parent() {
        if !ust.as_os_str().is_empty() {
            fs::create_dir_all(ust).map_err(|hata| yazma_hatasi(ust, hata))?;
        }
    }
    let metin =
        serde_json::to_string_pretty(icerik).map_err(|hata| TypeFastHata::AktarmaHatasi {
            yol: Some(dosya.to_path_buf()),
            ayrinti: hata.to_string(),
        })?;

    let gecici = gecici_yol(dosya);
    fs::write(&gecici, metin.as_bytes()).map_err(|hata| yazma_hatasi(&gecici, hata))?;
    if let Err(hata) = fs::rename(&gecici, dosya) {
        // Geçici dosya artık gereksiz; temizlik hatası kullanıcıya gösterilmez
        // çünkü asıl hata çok daha açıklayıcıdır.
        let _ = fs::remove_file(&gecici);
        return Err(yazma_hatasi(dosya, hata));
    }
    Ok(())
}

/// Geçici dosya yolunu üretir: `<ad>.tmp`.
///
/// Testlerde kullanıcı verisinin üzerine yazılmadığını doğrulamak için
/// `.gitignore` içinde `*.tmp` zaten vardır.
pub fn gecici_yol(dosya: &Path) -> PathBuf {
    let ad = dosya
        .file_name()
        .map(|a| a.to_string_lossy().to_string())
        .unwrap_or_else(|| "typefast".to_string());
    dosya.with_file_name(format!("{ad}.tmp"))
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::pano::PanoGecmisi;
    use std::path::PathBuf;

    /// Test içinde geçici dosya/dizin üreten, `Drop` ile temizleyen kapsayıcı.
    ///
    /// `tempfile` crate'i bağımlılık politikası gereği yasaktır
    /// (`WORKER_CONTRACT.md` § 5.3); yardımcı kendi kodumuzla yazılır.
    struct GeciciDizin {
        yol: PathBuf,
    }

    impl GeciciDizin {
        fn yeni(etiket: &str) -> Self {
            let kok =
                std::env::temp_dir().join(format!("typefast-{etiket}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&kok);
            fs::create_dir_all(&kok).expect("gecici dizin olusmali");
            Self { yol: kok }
        }

        fn yol(&self) -> &Path {
            &self.yol
        }
    }

    impl Drop for GeciciDizin {
        fn drop(&mut self) {
            // Temizlik basarisiz olsa da testi dusurmemeli; `let _ =` bilincli.
            let _ = fs::remove_dir_all(&self.yol);
        }
    }

    #[test]
    fn depo_yolu_dosya_adlarini_uretir() {
        let yol = DepoYolu::yeni("/tmp/x");
        assert_eq!(yol.sablon_dosyasi(), PathBuf::from("/tmp/x/sablonlar.json"));
        assert_eq!(yol.pano_dosyasi(), PathBuf::from("/tmp/x/pano.json"));
        assert_eq!(
            yol.istatistik_dosyasi(),
            PathBuf::from("/tmp/x/istatistik.json")
        );
    }

    #[test]
    fn yoldan_uzanti_varsa_kok_dizini_alir() {
        let yol = yoldan("/veri/sablonlar.json");
        assert_eq!(yol.kok, PathBuf::from("/veri"));
    }

    #[test]
    fn yoldan_uzanti_yoksa_kok_dizini_kendisi() {
        let yol = yoldan("/veri");
        assert_eq!(yol.kok, PathBuf::from("/veri"));
    }

    #[test]
    fn gecici_yol_tmp_ekine_sahip() {
        assert_eq!(
            gecici_yol(Path::new("/a/sablonlar.json")),
            PathBuf::from("/a/sablonlar.json.tmp")
        );
    }

    #[test]
    fn dosya_yoksa_bos_depo_donuyor() {
        let dizin = GeciciDizin::yeni("yok");
        let yol = DepoYolu::yeni(dizin.yol());
        let depo = sablonlari_yukle(&yol).expect("bos depo donmeli");
        assert!(depo.bos());
    }

    #[test]
    fn sablon_kaydet_ve_yukle_gidis_donusu() {
        let dizin = GeciciDizin::yeni("sablon-ro");
        let yol = DepoYolu::yeni(dizin.yol());
        let mut depo = SablonDeposu::yeni();
        depo.ekle(crate::sablon::Sablon::yeni(";t", "gövde"))
            .expect("eklenmeli");
        sablonlari_kaydet(&yol, &depo).expect("kaydedilmeli");
        let geri = sablonlari_yukle(&yol).expect("yuklenmeli");
        assert_eq!(depo, geri);
    }

    #[test]
    fn pano_kaydet_ve_yukle_gidis_donusu() {
        let dizin = GeciciDizin::yeni("pano-ro");
        let yol = DepoYolu::yeni(dizin.yol());
        let mut pano = PanoGecmisi::yeni(5, 50);
        pano.ekle("metin", 1);
        panoyu_kaydet(&yol, &pano).expect("kaydedilmeli");
        assert_eq!(panoyu_yukle(&yol).expect("yuklenmeli"), pano);
    }

    #[test]
    fn istatistik_kaydet_ve_yukle_gidis_donusu() {
        let dizin = GeciciDizin::yeni("ist-ro");
        let yol = DepoYolu::yeni(dizin.yol());
        let mut i = Istatistik::yeni();
        i.ekle(
            ";t",
            crate::kazanc::olc("abc", ";t", crate::kazanc::sabit_sure_ms(1), 1.0),
            1.0,
        );
        istatistigi_kaydet(&yol, &i).expect("kaydedilmeli");
        assert_eq!(istatistigi_yukle(&yol).expect("yuklenmeli"), i);
    }

    #[test]
    fn atomik_yazma_gecici_dosya_birakmiyor() {
        let dizin = GeciciDizin::yeni("atomik");
        let yol = DepoYolu::yeni(dizin.yol());
        sablonlari_kaydet(&yol, &SablonDeposu::yeni()).expect("kaydedilmeli");
        assert!(yol.sablon_dosyasi().exists());
        assert!(!gecici_yol(&yol.sablon_dosyasi()).exists());
    }

    #[test]
    fn atomik_yazma_hedefi_guncelliyor_eskisini_silmiyor() {
        let dizin = GeciciDizin::yeni("guncelle");
        let yol = DepoYolu::yeni(dizin.yol());
        let mut depo = SablonDeposu::yeni();
        depo.ekle(crate::sablon::Sablon::yeni("a", "1"))
            .expect("eklenmeli");
        sablonlari_kaydet(&yol, &depo).expect("kaydedilmeli");
        let ilk = fs::read_to_string(yol.sablon_dosyasi()).expect("okunmali");
        depo.ekle(crate::sablon::Sablon::yeni("b", "2"))
            .expect("eklenmeli");
        sablonlari_kaydet(&yol, &depo).expect("kaydedilmeli");
        let ikinci = fs::read_to_string(yol.sablon_dosyasi()).expect("okunmali");
        assert!(!ilk.contains("\"b\""));
        assert!(ikinci.contains("\"b\""));
    }

    #[test]
    fn bozuk_depo_hata_donderiyor() {
        let dizin = GeciciDizin::yeni("bozuk");
        let yol = DepoYolu::yeni(dizin.yol());
        fs::write(yol.sablon_dosyasi(), "{ bu json degil").expect("yazilmali");
        match sablonlari_yukle(&yol) {
            Err(TypeFastHata::BozukDepo { yol: h, ayrinti }) => {
                assert_eq!(h, yol.sablon_dosyasi());
                assert!(!ayrinti.is_empty());
            }
            diger => panic!("bozuk depo hatasi bekleniyordu: {diger:?}"),
        }
    }

    #[test]
    fn bos_dosya_bos_depo_olarak_yorumlanir() {
        let dizin = GeciciDizin::yeni("bos-dosya");
        let yol = DepoYolu::yeni(dizin.yol());
        fs::write(yol.sablon_dosyasi(), "   \n").expect("yazilmali");
        assert!(sablonlari_yukle(&yol).expect("bos depo donmeli").bos());
    }

    #[test]
    fn eksik_dizin_olusturulur() {
        let dizin = GeciciDizin::yeni("alt-dizin");
        let yol = DepoYolu::yeni(dizin.yol().join("a").join("b"));
        sablonlari_kaydet(&yol, &SablonDeposu::yeni()).expect("kaydedilmeli");
        assert!(yol.sablon_dosyasi().exists());
    }

    #[test]
    fn dizin_olmayan_yol_dosya_olarak_hata_verir() {
        let dizin = GeciciDizin::yeni("dosya-karisi");
        let dosya = dizin.yol().join("bir-dosya");
        fs::write(&dosya, "x").expect("yazilmali");
        let yol = DepoYolu::yeni(dosya.join("sablonlar.json"));
        assert!(sablonlari_kaydet(&yol, &SablonDeposu::yeni()).is_err());
    }

    #[test]
    fn yukle_or_bos_varsayilan_deger_donduruyor() {
        let dizin = GeciciDizin::yeni("varsayilan");
        let dosya = dizin.yol().join("yok.json");
        let deger: Istatistik = yukle_or_bos(&dosya).expect("yuklenmeli");
        assert!(deger.olculmedi());
    }

    #[test]
    fn okuma_hatasi_yolu_dogru_aktarir() {
        let dizin = GeciciDizin::yeni("okuma");
        let dosya = dizin.yol().join("klasor.json");
        fs::create_dir_all(&dosya).expect("dizin olusmali");
        let sonuc: Sonuc<Istatistik> = yukle_or_bos(&dosya);
        assert!(matches!(sonuc, Err(TypeFastHata::OkumaHatasi { .. })));
    }
}
