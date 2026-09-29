//! Dışa ve içe aktarma — JSON ile düz metin liste.
//!
//! İki biçim desteklenir ve ikisi de **gidiş-dönüşlüdür** (gövde, kategori,
//! açıklama, gizlilik işareti ve kullanım sayacı korunur):
//!
//! * **JSON** — tam belge; `serde_json` ile okunur.
//! * **Düz metin** — satır başına bir şablon, alanlar **Sekme** ile ayrılır
//!   ve `\` `\`n`, `\t` dizileriyle kaçırılır. Elle düzenlenebilir.
//!
//! **Pano geçmişi ve istatistik dışa aktarılmaz.** Rapor `b07` bunu açıkça
//! ister: "Geçmiş VE öğrenme paketin içine ALINMAZ." Dışa aktarılan yalnız
//! şablon koleksiyonudur; pano geçmişi hiçbir çıktıda görünmez.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hata::{Sonuc, TypeFastHata};
use crate::sablon::{Sablon, SablonDeposu, DEPO_SURUMU};

/// Dışa aktarma belgesinin zarfı; sürüm ve kaynak bilgisi taşır.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paket {
    /// Zarf sürümü.
    pub surum: u32,
    /// Üreten aracın adı.
    #[serde(default)]
    pub ureten: String,
    /// Aktarılan şablon koleksiyonu.
    pub sablonlar: SablonDeposu,
}

/// Düz metin biçiminde alanları ayıran karakter.
pub const AYRAC: char = '\t';

/// Düz metin başlığı; içe aktarımda tanınır ve atlanır.
pub const BASLIK: &str = "# typefast sablon listesi v1";

/// Aktarma biçimi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bicim {
    /// Tam JSON belgesi.
    Json,
    /// Sekmeyle ayrılmış düz metin liste.
    Metin,
}

impl Bicim {
    /// Adı (`--bicim` bayrağı değeri).
    pub fn ad(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Metin => "metin",
        }
    }

    /// Ad ile biçim üretir.
    ///
    /// # Hatalar
    ///
    /// Ad `json` ya da `metin` değilse [`TypeFastHata::AktarmaHatasi`] döner.
    pub fn ayikla(ad: &str) -> Sonuc<Self> {
        match ad {
            "json" => Ok(Self::Json),
            "metin" => Ok(Self::Metin),
            diger => Err(TypeFastHata::AktarmaHatasi {
                yol: None,
                ayrinti: format!("bilinmeyen biçim '{diger}'; 'json' veya 'metin' yazın"),
            }),
        }
    }
}

impl std::str::FromStr for Bicim {
    type Err = TypeFastHata;

    fn from_str(metin: &str) -> Result<Self, Self::Err> {
        Self::ayikla(metin)
    }
}

/// Şablon koleksiyonunu JSON paketine çevirir.
pub fn json_paket(depo: &SablonDeposu) -> Sonuc<String> {
    let paket = Paket {
        surum: DEPO_SURUMU,
        ureten: format!("typefast {}", env!("CARGO_PKG_VERSION")),
        sablonlar: depo.clone(),
    };
    serde_json::to_string_pretty(&paket).map_err(|hata| TypeFastHata::AktarmaHatasi {
        yol: None,
        ayrinti: hata.to_string(),
    })
}

/// JSON paketini çözümler.
///
/// # Hatalar
///
/// Belge okunamazsa veya şemaya uymazsa [`TypeFastHata::AktarmaHatasi`] döner.
pub fn json_oku(metin: &str) -> Sonuc<SablonDeposu> {
    let paket: Paket = serde_json::from_str(metin).map_err(|hata| TypeFastHata::AktarmaHatasi {
        yol: None,
        ayrinti: hata.to_string(),
    })?;
    Ok(paket.sablonlar)
}

/// Şablon koleksiyonunu düz metin listeye çevirir.
pub fn metin_liste(depo: &SablonDeposu) -> String {
    let mut satirlar = vec![BASLIK.to_string()];
    for sablon in depo.tumune() {
        satirlar.push(
            [
                kacis(&sablon.tetik),
                kacis(&sablon.kategori),
                kacis(&sablon.aciklama),
                if sablon.gizli {
                    "gizli".to_string()
                } else {
                    String::new()
                },
                sablon.kullanim.to_string(),
                kacis(&sablon.govde),
            ]
            .join(&AYRAC.to_string()),
        );
    }
    satirlar.join("\n")
}

/// Düz metin listeyi çözümler.
///
/// # Hatalar
///
/// Satır sayısı beş alandan azsa veya sayısal alanlar okunamazsa
/// [`TypeFastHata::AktarmaHatasi`] döner.
pub fn metin_oku(metin: &str) -> Sonuc<SablonDeposu> {
    let mut depo = SablonDeposu::yeni();
    for (no, satir) in metin.lines().enumerate() {
        let temiz = satir.trim_end_matches('\r');
        if temiz.trim().is_empty() || temiz.starts_with('#') {
            continue;
        }
        let alanlar: Vec<&str> = temiz.split(AYRAC).collect();
        if alanlar.len() < 6 {
            return Err(TypeFastHata::AktarmaHatasi {
                yol: None,
                ayrinti: format!(
                    "satir {}: {} alan bekleniyordu, {} bulundu",
                    no + 1,
                    6,
                    alanlar.len()
                ),
            });
        }
        let kullanim: u64 = alanlar[4]
            .trim()
            .parse()
            .map_err(|_| TypeFastHata::AktarmaHatasi {
                yol: None,
                ayrinti: format!("satir {}: kullanim sayisi okunamadi", no + 1),
            })?;
        let sablon = Sablon {
            tetik: kacisin_oku(alanlar[0]),
            kategori: kacisin_oku(alanlar[1]),
            aciklama: kacisin_oku(alanlar[2]),
            gizli: alanlar[3].trim() == "gizli",
            kullanim,
            govde: kacisin_oku(alanlar[5]),
        };
        depo.ekle(sablon)
            .map_err(|hata| TypeFastHata::AktarmaHatasi {
                yol: None,
                ayrinti: format!("satir {}: {hata}", no + 1),
            })?;
    }
    Ok(depo)
}

/// Seçilen biçime göre dışa aktarır.
pub fn disa_aktar(depo: &SablonDeposu, bicim: Bicim) -> Sonuc<String> {
    match bicim {
        Bicim::Json => json_paket(depo),
        Bicim::Metin => Ok(metin_liste(depo)),
    }
}

/// İçeriğe bakarak biçimi sezgiler.
///
/// İlk boş olmayan satır `{#` ile başlıyorsa düz metindir; değilse JSON'dur.
pub fn bicim_sezgile(metin: &str) -> Bicim {
    for satir in metin.lines() {
        if satir.trim().is_empty() {
            continue;
        }
        return if satir.starts_with('#') {
            Bicim::Metin
        } else {
            Bicim::Json
        };
    }
    Bicim::Json
}

/// Seçilen biçime göre içe aktarır.
pub fn ice_aktar(metin: &str, bicim: Bicim) -> Sonuc<SablonDeposu> {
    match bicim {
        Bicim::Json => json_oku(metin),
        Bicim::Metin => metin_oku(metin),
    }
}

/// Dosyaya dışa aktarır.
pub fn dosyaya_yaz(yol: &Path, icerik: &str) -> Sonuc<()> {
    crate::depo::atomik_yaz_dosya(yol, icerik.as_bytes())
}

fn kacis(metin: &str) -> String {
    metin
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn kacisin_oku(metin: &str) -> String {
    let mut sonuc = String::with_capacity(metin.len());
    let karakterler: Vec<char> = metin.chars().collect();
    let mut i = 0usize;
    while i < karakterler.len() {
        if karakterler[i] == '\\' && i + 1 < karakterler.len() {
            match karakterler[i + 1] {
                'n' => sonuc.push('\n'),
                't' => sonuc.push('\t'),
                '\\' => sonuc.push('\\'),
                diger => {
                    sonuc.push('\\');
                    sonuc.push(diger);
                }
            }
            i += 2;
        } else {
            sonuc.push(karakterler[i]);
            i += 1;
        }
    }
    sonuc
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn depo_kur() -> SablonDeposu {
        let mut depo = SablonDeposu::yeni();
        let mut a = Sablon::yeni(";tesekkur", "Merhaba, teşekkür ederim.");
        a.kategori = "iletisim".to_string();
        a.aciklama = " açılış cümlesi".to_string();
        a.kullanim = 7;
        let mut b = Sablon::yeni("sifre", "parola: gizli-deger");
        b.kategori = "guvenlik".to_string();
        b.gizli = true;
        let mut c = Sablon::yeni("not", "satir1\nsatir2\tsutun");
        c.aciklama = "sekme ve satir ici".to_string();
        depo.ekle(a).expect("a eklenmeli");
        depo.ekle(b).expect("b eklenmeli");
        depo.ekle(c).expect("c eklenmeli");
        depo
    }

    #[test]
    fn bicim_ayiklama_calisiyor() {
        assert_eq!(Bicim::ayikla("json").expect("json"), Bicim::Json);
        assert_eq!(Bicim::ayikla("metin").expect("metin"), Bicim::Metin);
        assert!(Bicim::ayikla("xml").is_err());
    }

    #[test]
    fn bicim_adlari_dogru() {
        assert_eq!(Bicim::Json.ad(), "json");
        assert_eq!(Bicim::Metin.ad(), "metin");
    }

    #[test]
    fn bicim_display_denemesi_calisiyor() {
        let bicim: Bicim = "metin".parse().expect("ayristirilmeli");
        assert_eq!(bicim, Bicim::Metin);
    }

    #[test]
    fn json_gidis_donusu_tum_alanlari_koruyor() {
        let depo = depo_kur();
        let metin = json_paket(&depo).expect("paket uretilmeli");
        let geri = json_oku(&metin).expect("paket okunmali");
        assert_eq!(depo, geri);
        assert_eq!(geri.bul(";tesekkur").map(|s| s.kullanim), Some(7));
        assert!(geri.bul("sifre").map(|s| s.gizli).unwrap_or(false));
    }

    #[test]
    fn metin_gidis_donusu_tum_alanlari_koruyor() {
        let depo = depo_kur();
        let metin = metin_liste(&depo);
        let geri = metin_oku(&metin).expect("liste okunmali");
        assert_eq!(depo.uzunluk(), geri.uzunluk());
        assert_eq!(
            geri.bul("not").map(|s| s.govde.as_str()),
            Some("satir1\nsatir2\tsutun")
        );
        assert_eq!(geri.bul("sifre").map(|s| s.gizli), Some(true));
        assert_eq!(geri.bul(";tesekkur").map(|s| s.kullanim), Some(7));
    }

    #[test]
    fn metin_listesi_baslik_ile_basliyor() {
        let depo = depo_kur();
        assert!(metin_liste(&depo).starts_with(BASLIK));
    }

    #[test]
    fn metin_listesi_satir_sayisi_sablon_sayisi_ile_artan() {
        let depo = depo_kur();
        let metin = metin_liste(&depo);
        assert_eq!(metin.lines().count(), depo.uzunluk() + 1);
    }

    #[test]
    fn ters_kacis_dikeyleri_korunuyor() {
        let mut depo = SablonDeposu::yeni();
        depo.ekle(Sablon::yeni("a", "yol: C:\\dosya\\ad"))
            .expect("eklenmeli");
        let metin = metin_liste(&depo);
        let geri = metin_oku(&metin).expect("liste okunmali");
        assert_eq!(
            geri.bul("a").map(|s| s.govde.as_str()),
            Some("yol: C:\\dosya\\ad")
        );
    }

    #[test]
    fn eksik_alanli_satir_hata_donderiyor() {
        let metin = "# typefast sablon listesi v1\na\tb\tc";
        let hata = metin_oku(metin).expect_err("hata bekleniyor");
        assert!(hata.to_string().contains("6 alan"));
    }

    #[test]
    fn okunamayan_sayisal_alan_hata_donderiyor() {
        let metin = "# typefast sablon listesi v1\na\tb\tc\td\txx\tg";
        let hata = metin_oku(metin).expect_err("hata bekleniyor");
        assert!(hata.to_string().contains("kullanim"));
    }

    #[test]
    fn cakisan_tetik_ice_aktarmada_hata_donderiyor() {
        let metin = "# typefast sablon listesi v1\na\t\t\td\t0\tbir\na\t\t\td\t0\tiki";
        assert!(metin_oku(metin).is_err());
    }

    #[test]
    fn bos_ve_yorum_satirlari_atlaniyor() {
        let metin = "# typefast sablon listesi v1\n\n# yorum\n\na\t\t\td\t0\tbir\n";
        let depo = metin_oku(metin).expect("liste okunmali");
        assert_eq!(depo.uzunluk(), 1);
    }

    #[test]
    fn bicim_sezgisi_dogru_calisiyor() {
        assert_eq!(bicim_sezgile("# typefast\n"), Bicim::Metin);
        assert_eq!(bicim_sezgile("{\"surum\":1}"), Bicim::Json);
        assert_eq!(bicim_sezgile("\n\n  \n"), Bicim::Json);
    }

    #[test]
    fn disa_aktarma_ve_ice_aktarma_ayni_bicimde_calisiyor() {
        let depo = depo_kur();
        for bicim in [Bicim::Json, Bicim::Metin] {
            let metin = disa_aktar(&depo, bicim).expect("disa aktarilmali");
            let geri = ice_aktar(&metin, bicim).expect("ice aktarilmali");
            assert_eq!(depo.uzunluk(), geri.uzunluk());
        }
    }

    #[test]
    fn bozuk_json_hata_donderiyor() {
        assert!(json_oku("{ bozuk").is_err());
    }

    #[test]
    fn disa_aktarma_pano_gecmisi_icer_mez() {
        let depo = depo_kur();
        let metin = json_paket(&depo).expect("paket uretilmeli");
        assert!(!metin.contains("pano"));
        assert!(!metin.contains("kayitlar"));
    }

    #[test]
    fn kacisin_oku_bilinmeyen_kacisi_koruyor() {
        assert_eq!(kacisin_oku("\\q"), "\\q");
        assert_eq!(kacisin_oku("a\\tb"), "a\tb");
    }
}
