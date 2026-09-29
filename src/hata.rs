//! Proje genelinde kullanılan hata tipi.
//!
//! Tek bir `enum` tanımlar ve `Display` uygulamasını elle yazar. `thiserror`
//! gibi bir türetme crate'i bağımlılık politikası
//! (`WORKER_CONTRACT.md` § 3.2) gereği kullanılamaz.
//!
//! Hata sınıfları ayrı tutulur: kullanıcı "depo bozuk" ile "kısayol çakıştı"
//! mesajlarını farklı ele alabilmelidir (rapor `b07` — Hata yönetimi).

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// TypeFast'ın tüm çalışma zamanı hatalarını taşıyan tip.
#[derive(Debug)]
#[non_exhaustive]
pub enum TypeFastHata {
    /// Depo, pano veya istatistik dosyası okunamadı.
    OkumaHatasi {
        /// Okunamayan dosyanın yolu.
        yol: PathBuf,
        /// Altta yatan işletim sistemi hatası.
        hata: io::Error,
    },
    /// Dosya yazılamadı (atomik yazma sırasında geçici dosya dahil).
    YazmaHatasi {
        /// Yazılamayan yol.
        yol: PathBuf,
        /// Altta yatan işletim sistemi hatası.
        hata: io::Error,
    },
    /// JSON belgesi okundu ama şemaya uymuyor.
    BozukDepo {
        /// Bozuk olan dosyanın yolu.
        yol: PathBuf,
        /// Ayrıştırıcının verdiği açıklama.
        ayrinti: String,
    },
    /// Kısayol tetiği sözdizimsel olarak geçersiz.
    GecersizTetik {
        /// Kullanıcının verdiği tetik.
        tetik: String,
        /// Neden geçersiz olduğuna dair açıklama.
        sebep: String,
    },
    /// Yeni tetik, depoda zaten var olan bir tetikle çakışıyor.
    TetikCakismasi {
        /// Eklenmek istenen tetik.
        tetik: String,
        /// Çakışan kaydın açıklaması.
        mevcut: String,
    },
    /// Depoda aranan tetik bulunamadı.
    BilinmeyenSablon {
        /// Aranan tetik.
        tetik: String,
    },
    /// `{{` açıldı ama eşleşen `}}` hiç kapatılmadı.
    KapanmamisYerTutucu {
        /// Hatanın metindeki bayt ofseti.
        konum: usize,
    },
    /// Yer tutucu adı çözümlenemedi (ne yerleşik ne de verilen değerlerde).
    BilinmeyenYerTutucu {
        /// Yer tutucunun adı.
        ad: String,
        /// Hatanın metindeki bayt ofseti.
        konum: usize,
    },
    /// İç içe şablon çağrısı derinlik sınırını aştı.
    DerinlikSiniri {
        /// Uygulanan üst sınır.
        sinir: usize,
    },
    /// Genişleme adımı tavanı aşıldı (döngü koruması).
    AdimSiniri {
        /// Uygulanan üst sınır.
        sinir: usize,
    },
    /// Aynı şablon, kendi çağrı zinciri içinde ikinci kez çağrıldı.
    Cevrim {
        /// Kendini çağıran şablonun tetiği.
        tetik: String,
    },
    /// Eşit uzunlukta birden fazla tetik eşleşti; belirsizlik çözülemedi.
    CozumBelirsiz {
        /// Girdi metni.
        metin: String,
        /// Birbirinden ayırt edilemeyen aday tetikler.
        adaylar: Vec<String>,
    },
    /// Dışa/içe aktarma belgesi okunamadı ya da çözümlenemedi.
    AktarmaHatasi {
        /// Belge yolu (bellek içi aktarmada boş olabilir).
        yol: Option<PathBuf>,
        /// Açıklama.
        ayrinti: String,
    },
}

impl fmt::Display for TypeFastHata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OkumaHatasi { yol, hata } => {
                write!(f, "'{}' okunamadı: {hata}", yol.display())
            }
            Self::YazmaHatasi { yol, hata } => {
                write!(f, "'{}' yazılamadı: {hata}", yol.display())
            }
            Self::BozukDepo { yol, ayrinti } => {
                write!(f, "'{}' bozuk: {ayrinti}", yol.display())
            }
            Self::GecersizTetik { tetik, sebep } => {
                write!(f, "geçersiz tetik '{tetik}': {sebep}")
            }
            Self::TetikCakismasi { tetik, mevcut } => {
                write!(f, "'{tetik}' zaten kullanılıyor ({mevcut})")
            }
            Self::BilinmeyenSablon { tetik } => write!(f, "kısayol bulunamadı: '{tetik}'"),
            Self::KapanmamisYerTutucu { konum } => {
                write!(
                    f,
                    "kapanmamış '{{{{' (bayt {konum}); metnin geri kalanı '{{{{}}}}' bekliyor"
                )
            }
            Self::BilinmeyenYerTutucu { ad, konum } => {
                write!(f, "bilinmeyen yer tutucu '{ad}' (bayt {konum})")
            }
            Self::DerinlikSiniri { sinir } => {
                write!(f, "iç içe genişleme derinlik sınırı aşıldı: {sinir}")
            }
            Self::AdimSiniri { sinir } => {
                write!(f, "genişleme adımı tavanı aşıldı: {sinir}")
            }
            Self::Cevrim { tetik } => {
                write!(
                    f,
                    "şablon '{tetik}' kendi çağrı zincirinde ikinci kez çağrıldı"
                )
            }
            Self::CozumBelirsiz { metin, adaylar } => {
                write!(
                    f,
                    "'{metin}' için birden fazla eşit öncelikli tetik var: {}",
                    adaylar.join(", ")
                )
            }
            Self::AktarmaHatasi { yol, ayrinti } => match yol {
                Some(y) => write!(f, "'{}' aktarılamadı: {ayrinti}", y.display()),
                None => write!(f, "aktarılamadı: {ayrinti}"),
            },
        }
    }
}

impl Error for TypeFastHata {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OkumaHatasi { hata, .. } | Self::YazmaHatasi { hata, .. } => Some(hata),
            _ => None,
        }
    }
}

/// Sonucu hata tipiyle birlikte döndüren kısaltma.
pub type Sonuc<T> = Result<T, TypeFastHata>;

/// `OkumaHatasi` üreten yardımcı; `?` operatörüyle birlikte kullanılır.
pub(crate) fn okuma_hatasi(yol: &Path, hata: io::Error) -> TypeFastHata {
    TypeFastHata::OkumaHatasi {
        yol: yol.to_path_buf(),
        hata,
    }
}

/// `YazmaHatasi` üreten yardımcı; `?` operatörüyle birlikte kullanılır.
pub(crate) fn yazma_hatasi(yol: &Path, hata: io::Error) -> TypeFastHata {
    TypeFastHata::YazmaHatasi {
        yol: yol.to_path_buf(),
        hata,
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

    fn gecici_bos_yol() -> PathBuf {
        PathBuf::from("test-yol.json")
    }

    #[test]
    fn gecersiz_tetik_hatasi_mesaj_iceriyor() {
        let hata = TypeFastHata::GecersizTetik {
            tetik: "".to_string(),
            sebep: "boş".to_string(),
        };
        let metin = hata.to_string();
        assert_eq!(metin, "geçersiz tetik '': boş");
    }

    #[test]
    fn kapanmamis_yer_tutucu_konumu_mesajda_geciyor() {
        let hata = TypeFastHata::KapanmamisYerTutucu { konum: 12 };
        assert!(hata.to_string().contains("12"));
    }

    #[test]
    fn cozum_belirsiz_adaylari_listeliyor() {
        let hata = TypeFastHata::CozumBelirsiz {
            metin: "is".to_string(),
            adaylar: vec!["is".to_string(), ",is".to_string()],
        };
        let metin = hata.to_string();
        assert!(metin.contains("is, ,is"));
    }

    #[test]
    fn io_hatasi_kaynak_olarak_baglaniyor() {
        let hata = TypeFastHata::OkumaHatasi {
            yol: gecici_bos_yol(),
            hata: io::Error::new(io::ErrorKind::NotFound, "yok"),
        };
        assert!(hata.source().is_some());
    }

    #[test]
    fn io_hatasi_disi_hata_kaynagi_yok() {
        let hata = TypeFastHata::Cevrim {
            tetik: "a".to_string(),
        };
        assert!(hata.source().is_none());
    }

    #[test]
    fn aktarma_hatasi_yol_varsa_yolu_yaziyor() {
        let hata = TypeFastHata::AktarmaHatasi {
            yol: Some(gecici_bos_yol()),
            ayrinti: "bozuk".to_string(),
        };
        assert!(hata.to_string().contains("test-yol.json"));
    }

    #[test]
    fn aktarma_hatasi_yol_yoksa_dosya_adi_yazmiyor() {
        let hata = TypeFastHata::AktarmaHatasi {
            yol: None,
            ayrinti: "bozuk".to_string(),
        };
        assert_eq!(hata.to_string(), "aktarılamadı: bozuk");
    }

    #[test]
    fn yardimcilar_dogru_varyant_uretiyor() {
        let okuma = okuma_hatasi(
            &gecici_bos_yol(),
            io::Error::new(io::ErrorKind::PermissionDenied, "yok"),
        );
        assert!(matches!(okuma, TypeFastHata::OkumaHatasi { .. }));

        let yazma = yazma_hatasi(
            &gecici_bos_yol(),
            io::Error::new(io::ErrorKind::PermissionDenied, "yok"),
        );
        assert!(matches!(yazma, TypeFastHata::YazmaHatasi { .. }));
    }
}
