//! Ölçülebilir kazanç istatistiği — gerçek tuş sayacı.
//!
//! Raporun "hissedilen hız" vaadi (`b01`, `b08`) ancak **sayıyla** karşılanır.
//! Buradaki protokol şudur ve `## Atıflar` ile README'de aynen yazılıdır:
//!
//! | Ölçüt | Tanım |
//! |---|---|
//! | `gerekli_tus` | Şablon yoksa kullanıcının gövde metnini elle yazmak için basacağı tuş sayısı = gövde karakter sayısı. |
//! | `gercek_tus` | Kısayolla genişletirken basılan tuş sayısı = tetik karakter sayısı + 1 (genişletme tuşu). |
//! | `kazanc` | `gerekli_tus - gercek_tus`; negatifse kısayol zarar vermiştir ve öyle raporlanır. |
//! | `sure` | `std::time::Instant` ile ölçülen gerçek genişleme süresi. |
//!
//! **Sayı uydurulmaz.** `Istatistik` dosyası boşsa `stats` komutu "ölçülmedi"
//! der ve sıfır göstermek yerine bunu açıkça söyler. Tuş sayısı bir
//! **varsayım**dır (Türkçe klavyede `ş` için üç tuş basılır); bu yüzden
//! `--tus_carpani` ile çarpan verilebilir ve varsayılan `1.0` belgelenmiştir.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Varsayılan tuş çarpanı: bir karakter = bir tuş vuruşu.
///
/// Bu bir **varsayımdır**; rapor `b08` "bu değer kullanıcı tarafından
/// düzeltilebilir olmalıdır" der. Türkçe klavyede `ğ` ve `ü` gibi harfler
/// birden çok tuşla yazıldığı için gerçek oran 1'in üzerindedir.
pub const VARSAYILAN_TUS_CARPANI: f64 = 1.0;

/// Tek bir genişletmenin kazanç ölçümü.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kazanc {
    /// Şablon olmasaydı gereken tuş vuruşu.
    pub gerekli_tus: u64,
    /// Kısayolla basılan gerçek tuş vuruşu.
    pub gercek_tus: u64,
    /// `gerekli_tus - gercek_tus`.
    pub kazanc: i64,
    /// Kazanılan karakter sayısı (çarpan uygulanmamış ham sayı).
    pub karakter: u64,
    /// Ölçülen gerçek süre.
    pub sure: Duration,
}

impl Kazanc {
    /// Kazanç negatifse kısayol zarar vermiştir.
    pub fn zararli(&self) -> bool {
        self.kazanc < 0
    }

    /// Ölçülen süreyi mikrosaniye cinsinden döner.
    pub fn sure_mikrosaniye(&self) -> u128 {
        self.sure.as_micros()
    }
}

/// Tek bir tetiğin birikmiş toplamları.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TetikKazanci {
    /// Kaç kez genişletildi.
    pub adet: u64,
    /// Birikmiş tuş vuruşu kazancı.
    pub kazanc: i64,
    /// Birikmiş karakter kazancı.
    pub karakter: i64,
}

/// Kalıcı kazanç istatistiği.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Istatistik {
    /// Belge sürüm damgası.
    #[serde(default)]
    pub surum: u32,
    /// Toplam genişletme sayısı.
    #[serde(default)]
    pub genisletme: u64,
    /// Toplam kazanılan tuş vuruşu.
    #[serde(default)]
    pub kazanilan_tus: i64,
    /// Toplam kazanılan karakter.
    #[serde(default)]
    pub kazanilan_karakter: i64,
    /// Birikmiş gerçek süre (saniye, kayan nokta).
    #[serde(default)]
    pub gecen_sn: f64,
    /// Ölçümde kullanılan tuş çarpanı; dosyada saklanır ki rapor
    /// sonradan değiştirildiğinde fark görünsün.
    #[serde(default)]
    pub tus_carpani: f64,
    /// Tetik bazlı birikimler.
    #[serde(default)]
    pub tetikler: BTreeMap<String, TetikKazanci>,
}

impl Istatistik {
    /// Boş (ölçülmemiş) istatistik üretir.
    pub fn yeni() -> Self {
        Self::default()
    }

    /// Hiç ölçüm yapılmamış mı?
    pub fn olculmedi(&self) -> bool {
        self.genisletme == 0
    }

    /// Tek bir ölçümü toplarlara ekler.
    pub fn ekle(&mut self, tetik: &str, kazanc: Kazanc, carpan: f64) {
        self.surum = crate::sablon::DEPO_SURUMU;
        self.genisletme = self.genisletme.saturating_add(1);
        self.tus_carpani = carpan;
        let tus_kazanci = (kazanc.kazanc as f64 * carpan).round() as i64;
        self.kazanilan_tus = self.kazanilan_tus.saturating_add(tus_kazanci);
        self.kazanilan_karakter = self
            .kazanilan_karakter
            .saturating_add(kazanc.karakter as i64);
        self.gecen_sn += kazanc.sure.as_secs_f64();

        let giris = self.tetikler.entry(tetik.to_string()).or_default();
        giris.adet = giris.adet.saturating_add(1);
        giris.kazanc = giris.kazanc.saturating_add(tus_kazanci);
        giris.karakter = giris.karakter.saturating_add(kazanc.karakter as i64);
    }

    /// Ortalama genişleme süresi (saniye); ölçüm yoksa `None`.
    pub fn ortalama_sn(&self) -> Option<f64> {
        if self.genisletme == 0 {
            return None;
        }
        Some(self.gecen_sn / self.genisletme as f64)
    }

    /// En çok kazandıran ilk `adet` tetik.
    pub fn en_cok_kazanan(&self, adet: usize) -> Vec<(String, TetikKazanci)> {
        let mut liste: Vec<(String, TetikKazanci)> =
            self.tetikler.iter().map(|(k, v)| (k.clone(), *v)).collect();
        liste.sort_by(|a, b| b.1.kazanc.cmp(&a.1.kazanc).then_with(|| a.0.cmp(&b.0)));
        liste.truncate(adet);
        liste
    }
}

/// Bir genişletmenin kazancını ölçer.
///
/// `carpan` bir karakterin kaç tuş vuruşu sayıldığını belirler; `1.0` dışında
/// bir değer verilirse ölçülen sayılar çarpılır.
pub fn olc(govde: &str, tetik: &str, sure: Duration, carpan: f64) -> Kazanc {
    let gerekli = govde.chars().count() as u64;
    let tetik_tusu = tetik.trim_end().chars().count() as u64 + 1;
    let kazanc = gerekli as i64 - tetik_tusu as i64;
    Kazanc {
        gerekli_tus: (gerekli as f64 * carpan).round() as u64,
        gercek_tus: tetik_tusu,
        kazanc: (kazanc as f64 * carpan).round() as i64,
        karakter: gerekli,
        sure,
    }
}

/// Testlerde kullanılan sabit süre üreticisi.
pub fn sabit_sure_ms(milisaniye: u64) -> Duration {
    Duration::from_millis(milisaniye)
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn kisa_tetik_uzun_govdede_pozitif_kazanc_veriyor() {
        let k = olc("abcdefghij", ";t", sabit_sure_ms(1), VARSAYILAN_TUS_CARPANI);
        assert_eq!(k.gerekli_tus, 10);
        assert_eq!(k.gercek_tus, 3);
        assert_eq!(k.kazanc, 7);
        assert_eq!(k.karakter, 10);
        assert!(!k.zararli());
    }

    #[test]
    fn uzun_tetik_kisa_govdede_zararli() {
        let k = olc(
            "ok",
            ";cok-uzun-bir-kisayol-adi",
            sabit_sure_ms(1),
            VARSAYILAN_TUS_CARPANI,
        );
        assert!(k.zararli());
        assert!(k.kazanc < 0);
    }

    #[test]
    fn tus_carpani_sayilari_carpiyor() {
        let k = olc("abcdefghij", "x", sabit_sure_ms(1), 2.0);
        assert_eq!(k.gerekli_tus, 20);
        assert_eq!(k.kazanc, 16);
    }

    #[test]
    fn sure_mikrosaniyeye_cevrilir() {
        let k = olc("abc", "x", sabit_sure_ms(3), 1.0);
        assert_eq!(k.sure_mikrosaniye(), 3000);
    }

    #[test]
    fn bos_istatistik_olculmedi_isaretliyor() {
        let i = Istatistik::yeni();
        assert!(i.olculmedi());
        assert_eq!(i.ortalama_sn(), None);
    }

    #[test]
    fn istatistik_ekleme_toplamlari_guncelliyor() {
        let mut i = Istatistik::yeni();
        i.ekle(
            ";t",
            olc("abcdefghijkl", ";t", sabit_sure_ms(2000), 1.0),
            1.0,
        );
        i.ekle(
            ";t",
            olc("abcdefghijkl", ";t", sabit_sure_ms(4000), 1.0),
            1.0,
        );
        assert!(!i.olculmedi());
        assert_eq!(i.genisletme, 2);
        assert_eq!(i.tetikler[";t"].adet, 2);
        assert_eq!(i.gecen_sn, 6.0);
    }

    #[test]
    fn ortalama_sure_hesaplaniyor() {
        let mut i = Istatistik::yeni();
        i.ekle("a", olc("abcd", "a", sabit_sure_ms(10000), 1.0), 1.0);
        i.ekle("b", olc("abcd", "b", sabit_sure_ms(20000), 1.0), 1.0);
        assert_eq!(i.ortalama_sn(), Some(15.0));
    }

    #[test]
    fn en_cok_kazanan_siralamasi_calisiyor() {
        let mut i = Istatistik::yeni();
        i.ekle("kisa", olc("abc", "kisa", sabit_sure_ms(1), 1.0), 1.0);
        i.ekle(
            "uzun",
            olc("abcdefghij", "uzun", sabit_sure_ms(1), 1.0),
            1.0,
        );
        let liste = i.en_cok_kazanan(2);
        assert_eq!(liste[0].0, "uzun");
        assert_eq!(liste.len(), 2);
        assert_eq!(i.en_cok_kazanan(1).len(), 1);
    }

    #[test]
    fn bos_istatistik_ile_siralama_bos_donderiyor() {
        assert!(Istatistik::yeni().en_cok_kazanan(5).is_empty());
    }

    #[test]
    fn istatistik_json_gidis_donusu_calisiyor() {
        let mut i = Istatistik::yeni();
        i.ekle(";t", olc("abcdefghij", ";t", sabit_sure_ms(7), 1.5), 1.5);
        let metin = serde_json::to_string(&i).expect("serilestirilmeli");
        let geri: Istatistik = serde_json::from_str(&metin).expect("ayristirilmeli");
        assert_eq!(i, geri);
        assert_eq!(geri.tus_carpani, 1.5);
    }

    #[test]
    fn bos_istatistik_json_gidis_donusu_calisiyor() {
        let i = Istatistik::yeni();
        let metin = serde_json::to_string(&i).expect("serilestirilmeli");
        let geri: Istatistik = serde_json::from_str(&metin).expect("ayristirilmeli");
        assert_eq!(i, geri);
    }

    #[test]
    fn tirnakli_karakter_sayimi_dogru() {
        let k = olc("ab\ncd\t", "x", sabit_sure_ms(1), 1.0);
        assert_eq!(k.karakter, 6);
    }

    #[test]
    fn cok_baytli_karakter_bir_tus_sayiliyor() {
        let k = olc("日本��", "x", sabit_sure_ms(1), 1.0);
        assert_eq!(k.karakter, 4);
    }
}
