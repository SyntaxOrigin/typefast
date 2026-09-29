//! Şablon genişletme motoru.
//!
//! Gövde metni taranır ve `{{ ... }}` çiftleri çözülür. Kurallar:
//!
//! * **Yerleşikler:** `{{tarih}}`, `{{saat}}`, `{{zaman}}`, `{{isim}}`, `{{bos}}`.
//!   Tarih ve saat **yalnız `std::time`** ile hesaplanır; `chrono`/`time`
//!   bağımlılıkları yasaktır (`WORKER_CONTRACT.md` § 3.2-F). Zaman dilimi
//!   daima UTC'dir ve bu README'de açıkça yazılıdır.
//! * **Kullanıcı değerleri:** `{{isim|Ayşe}}` — değer verilirse o, verilmezse
//!   `|` işaretinden sonraki tanım kullanılır.
//! * **İç içe şablon çağrısı:** `{{>tetik}}` — başka bir şablonu çağırır.
//! * **Derinlik sınırı** ve **adım tavanı** sonsuz döngüyü engeller; ayrıca
//!   çağrı yığınında aynı tetik ikinci kez görülürse döngü hatası döner.
//! * **Kaçırılmamış `{{` kuralı:** açılmış ama kapatılmamış bir `{{`
//!   hata üretir; sessizce düz metin sanılmaz.

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use crate::cozumle::{kucult_harf, Tetik};
use crate::hata::{Sonuc, TypeFastHata};
use crate::sablon::{Sablon, SablonDeposu};

/// Varsayılan iç içe çağrı derinlik sınırı.
pub const VARSAYILAN_MAKS_DERINLIK: usize = 8;

/// Varsayılan genişleme adımı tavanı.
pub const VARSAYILAN_MAKS_ADIM: usize = 200;

/// Gün içindeki saniye sayısı.
const GUN_SANIYE: i64 = 86_400;

/// Genişleme ayarları.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ayarlar {
    /// `{{isim}}` yerleşiğinin döndürdüğü kullanıcı adı.
    pub isim: String,
    /// İç içe çağrı derinlik sınırı.
    pub maks_derinlik: usize,
    /// Adım tavanı.
    pub maks_adim: usize,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            isim: "Kullanici".to_string(),
            maks_derinlik: VARSAYILAN_MAKS_DERINLIK,
            maks_adim: VARSAYILAN_MAKS_ADIM,
        }
    }
}

impl Ayarlar {
    /// Kullanıcı adı dışında her şeyi varsayılan bırakır.
    pub fn isimli(isim: &str) -> Self {
        Self {
            isim: isim.to_string(),
            ..Self::default()
        }
    }
}

/// Bir genişletmenin sonucu ve ölçüm için gereken sayaçlar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genisletme {
    /// Genişletilmiş metin.
    pub metin: String,
    /// Çözülen `{{ ... }}` sayısı (yerleşikler dahil).
    pub adim: usize,
    /// Ulaşılan en derin çağrı düzeyi.
    pub derinlik: usize,
    /// İç içe çağrıda kullanılan tetikler, çağrı sırasıyla.
    pub kullanilan: Vec<String>,
}

/// Zaman kaynağı elle verilebilen genişletme motoru.
///
/// Zamanın dışarıdan verilmesi testlerin deterministik olmasını sağlar
/// (`WORKER_CONTRACT.md` § 5.2): `SystemTime::now()` yalnız üretim
/// yolunda kullanılır ve testin kararında yer almaz.
#[derive(Debug, Clone)]
pub struct Motor {
    ayarlar: Ayarlar,
    zaman: SystemTime,
}

impl Default for Motor {
    fn default() -> Self {
        Self::yeni(Ayarlar::default())
    }
}

impl Motor {
    /// Ayarlarla motor kurar ve zaman kaynağı olarak sistem saatini kullanır.
    pub fn yeni(ayarlar: Ayarlar) -> Self {
        Self {
            ayarlar,
            zaman: SystemTime::now(),
        }
    }

    /// Zaman kaynağını elle verir; testlerde sabit an kullanılır.
    pub fn zamanli(ayarlar: Ayarlar, zaman: SystemTime) -> Self {
        Self { ayarlar, zaman }
    }

    /// Motorun ayarlarının kopyasını döner.
    pub fn ayarlar(&self) -> &Ayarlar {
        &self.ayarlar
    }

    /// Şablonu genişletir ve kullanıcı değeri geçmez.
    pub fn genislet(&self, depo: &SablonDeposu, sablon: &Sablon) -> Sonuc<Genisletme> {
        self.genislet_metin(depo, &sablon.govde, &BTreeMap::new())
    }

    /// Bir şablonu, verilen kullanıcı değerleriyle genişletir.
    pub fn genislet_sablon(
        &self,
        depo: &SablonDeposu,
        sablon: &Sablon,
        degerler: &BTreeMap<String, String>,
    ) -> Sonuc<Genisletme> {
        self.genislet_metin(depo, &sablon.govde, degerler)
    }

    /// Ham metni genişletir.
    ///
    /// # Hatalar
    ///
    /// Kapanmamış `{{`, bilinmeyen yer tutucu, döngü, derinlik sınırı veya adım
    /// tavanı aşımı hata döner.
    pub fn genislet_metin(
        &self,
        depo: &SablonDeposu,
        metin: &str,
        degerler: &BTreeMap<String, String>,
    ) -> Sonuc<Genisletme> {
        let kucuk_degerler = degerler
            .iter()
            .map(|(k, v)| (kucult_harf(k), v.clone()))
            .collect();
        let mut durum = GenislemeDurumu {
            adim: 0,
            en_derin: 0,
            kullanilan: Vec::new(),
        };
        let sonuc = self.tara(depo, metin, &kucuk_degerler, &mut Vec::new(), &mut durum, 0)?;
        Ok(Genisletme {
            metin: sonuc,
            adim: durum.adim,
            derinlik: durum.en_derin,
            kullanilan: durum.kullanilan,
        })
    }

    fn tara(
        &self,
        depo: &SablonDeposu,
        metin: &str,
        degerler: &BTreeMap<String, String>,
        yigin: &mut Vec<String>,
        durum: &mut GenislemeDurumu,
        derinlik: usize,
    ) -> Sonuc<String> {
        durum.en_derin = durum.en_derin.max(derinlik);
        let mut cikti = String::with_capacity(metin.len());
        let mut konum = 0usize;

        while konum < metin.len() {
            // Multi-byte karakterin ortasinda kalinmamak icin sinira kadar ilerle.
            while konum < metin.len() && !metin.is_char_boundary(konum) {
                konum += 1;
            }
            if konum >= metin.len() {
                break;
            }
            let kalan = metin.get(konum..).unwrap_or("");
            let Some(kayma) = kalan.find("{{") else {
                cikti.push_str(kalan);
                break;
            };
            let acilis = konum + kayma;
            cikti.push_str(metin.get(konum..acilis).unwrap_or(""));
            durum.adim += 1;
            if durum.adim > self.ayarlar.maks_adim {
                return Err(TypeFastHata::AdimSiniri {
                    sinir: self.ayarlar.maks_adim,
                });
            }

            let ic_kalan = metin.get(acilis + 2..).unwrap_or("");
            let Some(kapanma) = ic_kalan.find("}}") else {
                return Err(TypeFastHata::KapanmamisYerTutucu { konum: acilis });
            };
            let govde = ic_kalan.get(..kapanma).unwrap_or("").trim();
            let bitis = acilis + 2 + kapanma + 2;

            if let Some(hedef) = govde.strip_prefix('>') {
                cikti.push_str(&self.ic_caagri(depo, hedef, degerler, yigin, durum, derinlik)?);
            } else {
                cikti.push_str(&self.cozumle_yer_tutucu(govde, degerler, acilis)?);
            }
            konum = bitis;
        }
        Ok(cikti)
    }

    #[allow(clippy::too_many_arguments)]
    fn ic_caagri(
        &self,
        depo: &SablonDeposu,
        tetik: &str,
        degerler: &BTreeMap<String, String>,
        yigin: &mut Vec<String>,
        durum: &mut GenislemeDurumu,
        derinlik: usize,
    ) -> Sonuc<String> {
        let temiz = tetik.trim();
        if temiz.is_empty() {
            return Err(TypeFastHata::BilinmeyenSablon {
                tetik: tetik.to_string(),
            });
        }
        let ayrismis = Tetik::ayikla(temiz).map_err(|_| TypeFastHata::BilinmeyenSablon {
            tetik: temiz.to_string(),
        })?;
        let anahtar = depo
            .bul(&ayrismis.duz)
            .map(Sablon::tetik_anahtari)
            .unwrap_or_else(|| kucult_harf(&ayrismis.duz).replace(' ', ""));

        if yigin.contains(&anahtar) {
            return Err(TypeFastHata::Cevrim {
                tetik: temiz.to_string(),
            });
        }
        if derinlik + 1 > self.ayarlar.maks_derinlik {
            return Err(TypeFastHata::DerinlikSiniri {
                sinir: self.ayarlar.maks_derinlik,
            });
        }
        let hedef = depo
            .bul(&ayrismis.duz)
            .ok_or_else(|| TypeFastHata::BilinmeyenSablon {
                tetik: temiz.to_string(),
            })?;

        yigin.push(anahtar);
        durum.kullanilan.push(hedef.tetik.clone());
        let sonuc = self.tara(depo, &hedef.govde, degerler, yigin, durum, derinlik + 1);
        yigin.pop();
        sonuc
    }

    fn cozumle_yer_tutucu(
        &self,
        govde: &str,
        degerler: &BTreeMap<String, String>,
        konum: usize,
    ) -> Sonuc<String> {
        if govde.is_empty() {
            return Err(TypeFastHata::BilinmeyenYerTutucu {
                ad: String::new(),
                konum,
            });
        }
        let (ad, tanim) = match govde.split_once('|') {
            Some((a, b)) => (a.trim(), Some(b)),
            None => (govde, None),
        };
        let anahtar = kucult_harf(ad);

        if let Some(deger) = self.yerlesik(&anahtar) {
            return Ok(deger);
        }
        if let Some(deger) = degerler_get(degerler, &anahtar) {
            return Ok(deger);
        }
        if let Some(tanim) = tanim {
            return Ok(tanim.to_string());
        }
        Err(TypeFastHata::BilinmeyenYerTutucu {
            ad: ad.to_string(),
            konum,
        })
    }

    /// Yerlesik yer tutucularin bugunku degerini doner.
    ///
    /// `None`, adin yerlesik olmadigi anlamina gelir.
    pub fn yerlesik(&self, ad: &str) -> Option<String> {
        let (yil, ay, gun) = self.tarih_bolumleri();
        let (saat, dakika, saniye) = self.saat_bolumleri();
        match ad {
            "tarih" => Some(format!("{yil:04}-{ay:02}-{gun:02}")),
            "saat" => Some(format!("{saat:02}:{dakika:02}:{saniye:02}")),
            "zaman" => Some(format!(
                "{yil:04}-{ay:02}-{gun:02}T{saat:02}:{dakika:02}:{saniye:02}Z"
            )),
            "isim" => Some(self.ayarlar.isim.clone()),
            "bos" => Some(String::new()),
            _ => None,
        }
    }

    fn epoch_saniye(&self) -> i64 {
        match self.zaman.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(gecen) => gecen.as_secs() as i64,
            Err(hata) => -(hata.duration().as_secs() as i64),
        }
    }

    /// `(yil, ay, gun)` ulsunu doner (UTC).
    pub fn tarih_bolumleri(&self) -> (i64, u32, u32) {
        gun_sayisindan_tarihe(self.epoch_saniye().div_euclid(GUN_SANIYE))
    }

    /// `(saat, dakika, saniye)` ulsunu doner (UTC).
    pub fn saat_bolumleri(&self) -> (u64, u64, u64) {
        let kalan = self.epoch_saniye().rem_euclid(GUN_SANIYE) as u64;
        (kalan / 3600, (kalan % 3600) / 60, kalan % 60)
    }

    /// Ay adlarini doner. Yerlesikler sayisal oldugu icin bu liste yalnizca
    /// belge ve arayuz metinlerinde kullanilir.
    pub fn ay_adlari() -> [&'static str; 12] {
        [
            "ocak", "subat", "mart", "nisan", "mayis", "haziran", "temmuz", "agustos", "eylul",
            "ekim", "kasim", "aralik",
        ]
    }
}

struct GenislemeDurumu {
    adim: usize,
    en_derin: usize,
    kullanilan: Vec<String>,
}

fn degerler_get(degerler: &BTreeMap<String, String>, anahtar: &str) -> Option<String> {
    degerler.get(anahtar).cloned()
}

/// Olceklenmis bayt tabanli takvim donusumu (Howard Hinnant, `civil_from_days`).
///
/// Bu algoritma kamu malidir ve Hinnant'in "chrono-compatible Low-Level Date
/// Algorithms" yazisindan alinmistir; `README.md` -> `## Atiflar` bolumunde
/// kaynak olarak verilir.
pub fn gun_sayisindan_tarihe(gun: i64) -> (i64, u32, u32) {
    let z = gun + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let yil = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let gun = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let ay = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if ay <= 2 { yil + 1 } else { yil }, ay, gun)
}

/// Unix saniyesinden `(yil, ay, gun, saat, dakika, saniye)` ulsunu uretir.
pub fn epoch_bolumleri(epoch: i64) -> (i64, u32, u32, u64, u64, u64) {
    let (yil, ay, gun) = gun_sayisindan_tarihe(epoch.div_euclid(GUN_SANIYE));
    let kalan = epoch.rem_euclid(GUN_SANIYE) as u64;
    (yil, ay, gun, kalan / 3600, (kalan % 3600) / 60, kalan % 60)
}

/// Test ve toplu isleme icin sabit bir an uretir.
pub fn sabit_an(epoch_saniye: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(epoch_saniye)
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// 2026-09-29T14:05:09Z - test vektoru.
    const VEKTOR: u64 = 1_790_690_709;

    fn motor() -> Motor {
        Motor::zamanli(Ayarlar::isimli("Ceren"), sabit_an(VEKTOR))
    }

    fn depo_kur(cesitler: &[(&str, &str)]) -> SablonDeposu {
        let mut depo = SablonDeposu::yeni();
        for (tetik, govde) in cesitler {
            depo.ekle(Sablon::yeni(tetik, govde)).expect("eklenmeli");
        }
        depo
    }

    #[test]
    fn epoch_donusumu_epoch_sifirini_dogru_veriyor() {
        let (yil, ay, gun, saat, dakika, saniye) = epoch_bolumleri(0);
        assert_eq!((yil, ay, gun), (1970, 1, 1));
        assert_eq!((saat, dakika, saniye), (0, 0, 0));
    }

    #[test]
    fn epoch_donusumu_katlanma_gununu_dogru_veriyor() {
        // 2024-02-29 dort yilda bir artan gun.
        let (yil, ay, gun) = gun_sayisindan_tarihe(19_782);
        assert_eq!((yil, ay, gun), (2024, 2, 29));
    }

    #[test]
    fn epoch_donusumu_yuzyil_katlamasi_dogru() {
        // 2000-03-01
        let (yil, ay, gun) = gun_sayisindan_tarihe(11_017);
        assert_eq!((yil, ay, gun), (2000, 3, 1));
    }

    #[test]
    fn motor_test_vektorunu_tarih_ve_saat_olarak_veriyor() {
        let (yil, ay, gun) = motor().tarih_bolumleri();
        assert_eq!((yil, ay, gun), (2026, 9, 29));
        assert_eq!(motor().saat_bolumleri(), (14, 5, 9));
    }

    #[test]
    fn yerlesik_tarih_saat_zaman_bicimleri_dogru() {
        let m = motor();
        assert_eq!(m.yerlesik("tarih").as_deref(), Some("2026-09-29"));
        assert_eq!(m.yerlesik("saat").as_deref(), Some("14:05:09"));
        assert_eq!(m.yerlesik("zaman").as_deref(), Some("2026-09-29T14:05:09Z"));
    }

    #[test]
    fn yerlesik_isim_ayarlardan_geliyor() {
        assert_eq!(motor().yerlesik("isim").as_deref(), Some("Ceren"));
    }

    #[test]
    fn yerlesik_bos_bos_dize_donderiyor() {
        assert_eq!(motor().yerlesik("bos").as_deref(), Some(""));
    }

    #[test]
    fn yerlesik_olmayan_ad_none_donderiyor() {
        assert!(motor().yerlesik("olmayan").is_none());
    }

    #[test]
    fn ay_adlari_on_iki_tane() {
        assert_eq!(Motor::ay_adlari().len(), 12);
        assert_eq!(Motor::ay_adlari()[8], "eylul");
    }

    #[test]
    fn duz_metin_degistirilmeden_geciyor() {
        let depo = SablonDeposu::yeni();
        let sonuc = motor().genislet_metin(&depo, "Merhaba {{isim}}!", &BTreeMap::new());
        let g = sonuc.expect("genislemeli");
        assert_eq!(g.metin, "Merhaba Ceren!");
        assert_eq!(g.adim, 1);
    }

    #[test]
    fn yerlesik_adi_kullanici_degeri_eziyor() {
        let depo = SablonDeposu::yeni();
        let degerler = BTreeMap::from([("isim".to_string(), "Mert".to_string())]);
        let g = motor()
            .genislet_metin(&depo, "{{isim}}", &degerler)
            .expect("genislemeli");
        assert_eq!(g.metin, "Ceren");
    }

    #[test]
    fn kullanici_degeri_anahtari_harf_duyarsiz() {
        let depo = SablonDeposu::yeni();
        let degerler = BTreeMap::from([("Konu".to_string(), "Fatura".to_string())]);
        let g = motor()
            .genislet_metin(&depo, "{{KONU}}", &degerler)
            .expect("genislemeli");
        assert_eq!(g.metin, "Fatura");
    }

    #[test]
    fn tanimli_yer_tutucu_varsayilani_kullaniyor() {
        let depo = SablonDeposu::yeni();
        let g = motor()
            .genislet_metin(
                &depo,
                "Sayin {{musteri|Kiyemetli Musteri}},",
                &BTreeMap::new(),
            )
            .expect("genislemeli");
        assert_eq!(g.metin, "Sayin Kiyemetli Musteri,");
    }

    #[test]
    fn tanimli_yer_tutucu_verilen_degeri_kullaniyor() {
        let depo = SablonDeposu::yeni();
        let degerler = BTreeMap::from([("musteri".to_string(), "Ayse".to_string())]);
        let g = motor()
            .genislet_metin(&depo, "Sayin {{musteri|Kiyemetli}},", &degerler)
            .expect("genislemeli");
        assert_eq!(g.metin, "Sayin Ayse,");
    }

    #[test]
    fn kapanmamis_yer_tutucu_hata_donderiyor() {
        let depo = SablonDeposu::yeni();
        let hata = motor()
            .genislet_metin(&depo, "selam {{tarih", &BTreeMap::new())
            .expect_err("hata bekleniyor");
        match hata {
            TypeFastHata::KapanmamisYerTutucu { konum } => assert_eq!(konum, 6),
            diger => panic!("beklenmeyen hata: {diger:?}"),
        }
    }

    #[test]
    fn kapanmamis_yer_tutucu_sonda_da_hata_donderiyor() {
        let depo = SablonDeposu::yeni();
        assert!(matches!(
            motor().genislet_metin(&depo, "{{", &BTreeMap::new()),
            Err(TypeFastHata::KapanmamisYerTutucu { .. })
        ));
    }

    #[test]
    fn bilinmeyen_yer_tutucu_hata_donderiyor() {
        let depo = SablonDeposu::yeni();
        match motor().genislet_metin(&depo, "merhaba {{bilinmeyen}}", &BTreeMap::new()) {
            Err(TypeFastHata::BilinmeyenYerTutucu { ad, .. }) => assert_eq!(ad, "bilinmeyen"),
            diger => panic!("beklenmeyen hata: {diger:?}"),
        }
    }

    #[test]
    fn bos_yer_tutucu_hata_donderiyor() {
        let depo = SablonDeposu::yeni();
        assert!(matches!(
            motor().genislet_metin(&depo, "{{}}", &BTreeMap::new()),
            Err(TypeFastHata::BilinmeyenYerTutucu { .. })
        ));
    }

    #[test]
    fn ic_ice_sablon_cagrisi_calisiyor() {
        let depo = depo_kur(&[
            ("karsilama", "Sayin {{musteri|Musteri}},"),
            ("selam", "{{>karsilama}}"),
        ]);
        let sablon = depo.bul("selam").expect("sablon olmali");
        let g = motor().genislet(&depo, sablon).expect("genislemeli");
        assert_eq!(g.metin, "Sayin Musteri,");
        assert_eq!(g.kullanilan, vec!["karsilama"]);
        assert_eq!(g.derinlik, 1);
    }

    #[test]
    fn kendini_cagrilan_sablon_cevrim_hatasi_donderiyor() {
        let depo = depo_kur(&[("a", "x {{>a}} y")]);
        assert!(matches!(
            motor().genislet_metin(&depo, "{{>a}}", &BTreeMap::new()),
            Err(TypeFastHata::Cevrim { .. })
        ));
    }

    #[test]
    fn iki_dugumlu_cevrim_da_yakalaniyor() {
        let depo = depo_kur(&[("a", "{{>b}}"), ("b", "{{>a}}")]);
        assert!(matches!(
            motor().genislet_metin(&depo, "{{>a}}", &BTreeMap::new()),
            Err(TypeFastHata::Cevrim { .. })
        ));
    }

    #[test]
    fn derinlik_siniri_asiminda_hata_donderiliyor() {
        let mut depo = SablonDeposu::yeni();
        for (ad, govde) in [
            ("s0", "{{>s1}} son"),
            ("s1", "{{>s2}} son"),
            ("s2", "{{>s3}} son"),
            ("s3", "{{>s4}} son"),
            ("s4", "son"),
        ] {
            depo.ekle(Sablon::yeni(ad, govde)).expect("eklenmeli");
        }
        let ayarlar = Ayarlar {
            maks_derinlik: 2,
            ..Ayarlar::isimli("Ceren")
        };
        let m = Motor::zamanli(ayarlar, sabit_an(VEKTOR));
        match m.genislet_metin(&depo, "{{>s0}}", &BTreeMap::new()) {
            Err(TypeFastHata::DerinlikSiniri { sinir }) => assert_eq!(sinir, 2),
            diger => panic!("derinlik hatasi bekleniyordu: {diger:?}"),
        }
    }

    #[test]
    fn derinlik_siniri_ici_cagri_geciyor() {
        let depo = depo_kur(&[("a", "{{>b}}"), ("b", "son")]);
        let g = motor()
            .genislet_metin(&depo, "{{>a}}", &BTreeMap::new())
            .expect("genislemeli");
        assert_eq!(g.metin, "son");
        assert_eq!(g.derinlik, 2);
    }

    #[test]
    fn adim_tavani_asiminda_hata_donderiliyor() {
        let depo = SablonDeposu::yeni();
        let ayarlar = Ayarlar {
            maks_adim: 3,
            ..Ayarlar::isimli("Ceren")
        };
        let m = Motor::zamanli(ayarlar, sabit_an(VEKTOR));
        let metin = "{{bos}}{{bos}}{{bos}}{{bos}}";
        match m.genislet_metin(&depo, metin, &BTreeMap::new()) {
            Err(TypeFastHata::AdimSiniri { sinir }) => assert_eq!(sinir, 3),
            diger => panic!("adim hatasi bekleniyordu: {diger:?}"),
        }
    }

    #[test]
    fn adim_tavani_esitte_gecme_izin_veriyor() {
        let depo = SablonDeposu::yeni();
        let ayarlar = Ayarlar {
            maks_adim: 3,
            ..Ayarlar::isimli("Ceren")
        };
        let m = Motor::zamanli(ayarlar, sabit_an(VEKTOR));
        let metin = "{{bos}}{{bos}}{{bos}}";
        let g = m
            .genislet_metin(&depo, metin, &BTreeMap::new())
            .expect("genislemeli");
        assert_eq!(g.adim, 3);
    }

    #[test]
    fn var_olmayan_ic_cagri_bilinmeyen_sablon_hatasi_donderiyor() {
        let depo = SablonDeposu::yeni();
        assert!(matches!(
            motor().genislet_metin(&depo, "{{>yok}}", &BTreeMap::new()),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn bos_ic_cagri_bilinmeyen_sablon_hatasi_donderiyor() {
        let depo = SablonDeposu::yeni();
        assert!(matches!(
            motor().genislet_metin(&depo, "{{>  }}", &BTreeMap::new()),
            Err(TypeFastHata::BilinmeyenSablon { .. })
        ));
    }

    #[test]
    fn sablon_genisletme_yardimcisi_calisiyor() {
        let depo = depo_kur(&[("selam", "Merhaba {{isim}}")]);
        let sablon = depo.bul("selam").expect("sablon olmali");
        let g = motor().genislet(&depo, sablon).expect("genislemeli");
        assert_eq!(g.metin, "Merhaba Ceren");
    }

    #[test]
    fn sablon_degerli_genisletme_yardimcisi_calisiyor() {
        let depo = depo_kur(&[("selam", "Sayin {{musteri|M.}}")]);
        let sablon = depo.bul("selam").expect("sablon olmali");
        let degerler = BTreeMap::from([("musteri".to_string(), "Mert".to_string())]);
        let g = motor()
            .genislet_sablon(&depo, sablon, &degerler)
            .expect("genislemeli");
        assert_eq!(g.metin, "Sayin Mert");
    }

    #[test]
    fn varsayilan_ayarlar_ve_motor_calisiyor() {
        let m = Motor::default();
        assert_eq!(m.ayarlar().isim, "Kullanici");
        assert_eq!(m.ayarlar().maks_derinlik, VARSAYILAN_MAKS_DERINLIK);
        assert_eq!(m.ayarlar().maks_adim, VARSAYILAN_MAKS_ADIM);
        let depo = SablonDeposu::yeni();
        let g = m
            .genislet_metin(&depo, "{{isim}}", &BTreeMap::new())
            .expect("genislemeli");
        assert_eq!(g.metin, "Kullanici");
    }

    #[test]
    fn cok_baytli_karakter_ardindan_tarama_kaymadan_gidiyor() {
        let depo = SablonDeposu::yeni();
        let g = motor()
            .genislet_metin(&depo, "AA {{isim}} AA", &BTreeMap::new())
            .expect("genislemeli");
        assert_eq!(g.metin, "AA Ceren AA");
    }

    #[test]
    fn turkce_karakter_ardindan_tarama_kaymadan_gidiyor() {
        let depo = SablonDeposu::yeni();
        let g = motor()
            .genislet_metin(&depo, "Islem isikli {{isim}}", &BTreeMap::new())
            .expect("genislemeli");
        assert_eq!(g.metin, "Islem isikli Ceren");
    }

    #[test]
    fn negatif_epoch_bolumleri_pozitif_mod_aliyor() {
        // 1969-12-31T23:59:59Z
        let (yil, ay, gun, saat, dakika, saniye) = epoch_bolumleri(-1);
        assert_eq!((yil, ay, gun), (1969, 12, 31));
        assert_eq!((saat, dakika, saniye), (23, 59, 59));
    }

    #[test]
    fn degerler_get_anahtari_buluyor() {
        let degerler = BTreeMap::from([("a".to_string(), "b".to_string())]);
        assert_eq!(degerler_get(&degerler, "a").as_deref(), Some("b"));
        assert!(degerler_get(&degerler, "z").is_none());
    }
}
