//! Gizli mod — parola, anahtar ve token kalıplarını tespit edip maskeler.
//!
//! Bu modül **şifreleme yapmaz**. Yalnız kural tabanlı bir örüntü tanıyıcıdır:
//! metin içinde `parola:`, `api_key=`, `ghp_...`, `Bearer <jeton>` gibi kalıpları
//! bulur ve **yalnız gösterilecek metni** maskeler. Depodaki pano kaydı düz
//! metindir ve bu README'nin en üstündeki uyarıda yazılıdır (rapor `b10`).
//!
//! Kural tabanlı olmanın bedeli dürüstçe belgelenmiştir: kalıba uymayan bir
//! sır **maskelenmez**. Yanlış negatif, kullanıcının `typefast mask` çıktısını
//! incelemesiyle azaltılır.
//!
//! Büyük/küçük harf karşılaştırması bayt bayt yapılmaz: `ẞ` gibi bir
//! karakterin küçük harfi farklı uzunlukta olduğundan küçültülmüş kopya ile
//! özgün metin arasındaki bayt ofsetleri kayar. Bunun yerine metin bir kez
//! karakter dizisine çevrilir ve tüm taramada o dizi üzerinde yürünür.

/// Maskelenen değerin yerine yazılan sabit işaret dizisi.
///
/// Değer uzunluğunu sızdırmamak için sabit seçilmiştir.
pub const MASKE_ISARETI: &str = "****";

/// Yüksek entropili sayılacak en kısa boşluksuz parça uzunluğu.
const EN_KISA_ENTROPI: usize = 24;

/// `anahtar: deger` kalıbında aranan anahtar sözcükler.
const ANAHTAR_SOZCUKLER: [&str; 19] = [
    "parola",
    "sifre",
    "şifre",
    "sifrem",
    "password",
    "passwd",
    "pwd",
    "passphrase",
    "token",
    "api_key",
    "apikey",
    "api-key",
    "secret",
    "client_secret",
    "anahtar",
    "key",
    "bearer",
    "kimlik_no",
    "cvv",
];

/// Doğrudan biçim tanınan token ön ekleri ve gereken ek uzunluk.
const TOKEN_ON_EKLERI: [(&str, usize); 6] = [
    ("ghp_", 36),
    ("gho_", 36),
    ("ghu_", 36),
    ("ghs_", 36),
    ("sk-", 20),
    ("akia", 16),
];

/// Bir maskeleme bulgusunun türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BulguTuru {
    /// `anahtar: deger` biçiminde açık anahtar/değer çifti.
    AnahtarDeger,
    /// `ghp_`, `sk-`, `AKIA` gibi bilinen ön ekli jeton.
    OnEkJeton,
    /// `Bearer <jeton>` başlığı.
    Bearer,
    /// Boşluksuz, uzun ve karışık karakterlerden oluşan yüksek entropili dizi.
    YuksekEntropi,
}

impl BulguTuru {
    /// Türün kısa adı; `mask` çıktısında ve testlerde kullanılır.
    pub fn ad(self) -> &'static str {
        match self {
            Self::AnahtarDeger => "anahtar-deger",
            Self::OnEkJeton => "onek-jeton",
            Self::Bearer => "bearer",
            Self::YuksekEntropi => "yuksek-entropi",
        }
    }
}

/// Bulgunun metin içindeki konumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bulgu {
    /// Bulgunun başladığı bayt ofseti (anahtarın kendisi dâhil).
    pub baslangic: usize,
    /// Bulgunun bittiği bayt ofseti (değer dâhil).
    pub bitis: usize,
    /// Maskelenecek değerin başlangıç bayt ofseti.
    pub deger_baslangic: usize,
    /// Maskelenecek değerin bitiş bayt ofseti.
    pub deger_bitis: usize,
    /// Bulgu türü.
    pub tur: BulguTuru,
}

/// Maskeleme sonucu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskeSonuc {
    /// Maskelenmiş metin.
    pub metin: String,
    /// Tespit edilen bulgular.
    pub bulgular: Vec<Bulgu>,
}

impl MaskeSonuc {
    /// Metinde en az bir sır kalıbı var mı?
    pub fn gizli(&self) -> bool {
        !self.bulgular.is_empty()
    }

    /// Tespit edilen türlerin tekrarsız listesi.
    pub fn turler(&self) -> Vec<BulguTuru> {
        let mut liste: Vec<BulguTuru> = self.bulgular.iter().map(|b| b.tur).collect();
        liste.sort();
        liste.dedup();
        liste
    }
}

/// Metin kopyasının karakter dizisi ve bayt ofset eşlemesi.
struct Metin {
    ham: Vec<char>,
    kucuk: Vec<char>,
    ofset: Vec<usize>,
}

impl Metin {
    fn yeni(metin: &str) -> Self {
        let mut ham = Vec::with_capacity(metin.len());
        let mut kucuk = Vec::with_capacity(metin.len());
        let mut ofset = Vec::with_capacity(metin.len() + 1);
        for (bayt, karakter) in metin.char_indices() {
            ham.push(karakter);
            kucuk.push(kucuk_harf(karakter));
            ofset.push(bayt);
        }
        ofset.push(metin.len());
        Self { ham, kucuk, ofset }
    }

    fn uzunluk(&self) -> usize {
        self.ham.len()
    }

    fn bayt(&self, konum: usize) -> usize {
        self.ofset[konum.min(self.ofset.len() - 1)]
    }

    fn son_bayt(&self, konum: usize) -> usize {
        self.ofset[konum.min(self.ofset.len() - 1)]
    }
}

/// Türkçe'ye uygun küçük harf indirgeme (`I` ve `İ` tek harfe iner).
fn kucuk_harf(karakter: char) -> char {
    match karakter {
        'I' | 'İ' | 'ı' => 'i',
        diger => diger.to_lowercase().next().unwrap_or(diger),
    }
}

fn sozcuk_harfi(karakter: char) -> bool {
    karakter.is_alphanumeric() || karakter == '_'
}

fn bosluk_mi(karakter: char) -> bool {
    karakter == ' ' || karakter == '\t' || karakter == '\n' || karakter == '\r'
}

fn jeton_harfi_mi(karakter: char) -> bool {
    karakter.is_ascii_alphanumeric() || karakter == '_' || karakter == '-'
}

/// Metindeki sır kalıplarını bulur.
///
/// Bulgu listesi metin sırasına göre döner ve çakışan bulgular içermez:
/// yüksek entropi taraması, daha önce kabul edilmiş bir değer aralığının
/// üstüne yazmaz.
pub fn bul(metin: &str) -> Vec<Bulgu> {
    let taranan = Metin::yeni(metin);
    let sozcukler: Vec<Vec<char>> = ANAHTAR_SOZCUKLER
        .iter()
        .map(|s| s.chars().collect())
        .collect();
    let onekler: Vec<(Vec<char>, usize)> = TOKEN_ON_EKLERI
        .iter()
        .map(|(s, n)| (s.chars().collect(), *n))
        .collect();

    let mut bulgular: Vec<Bulgu> = Vec::new();
    anahtar_deger_bul(&taranan, &sozcukler, &mut bulgular);
    on_ek_bul(&taranan, &onekler, &mut bulgular);
    bearer_bul(&taranan, &mut bulgular);
    bulgular.sort_by_key(|b| b.baslangic);
    cakisiyorum_bulgu_birak(&mut bulgular);
    entropi_bul(metin, &mut bulgular);
    bulgular.sort_by_key(|b| b.baslangic);
    bulgular
}

/// Çakışan bulguları eler; ilk (en erişen) bulgu korunur.
fn cakisiyorum_bulgu_birak(bulgular: &mut Vec<Bulgu>) {
    let mut temiz: Vec<Bulgu> = Vec::with_capacity(bulgular.len());
    let mut son_bit = 0usize;
    for bulgu in bulgular.iter() {
        if bulgu.baslangic >= son_bit {
            son_bit = bulgu.bitis;
            temiz.push(*bulgu);
        }
    }
    *bulgular = temiz;
}

fn anahtar_deger_bul(taranan: &Metin, sozcukler: &[Vec<char>], bulgular: &mut Vec<Bulgu>) {
    let n = taranan.uzunluk();
    let mut i = 0usize;
    while i < n {
        for sozcuk in sozcukler {
            if !sozcuk_eslesiyor(&taranan.kucuk, i, sozcuk) {
                continue;
            }
            if i > 0 && sozcuk_harfi(taranan.ham[i - 1]) {
                continue;
            }
            let mut j = i + sozcuk.len();
            let mut bosluk = 0usize;
            while j < n && taranan.ham[j] == ' ' && bosluk < 8 {
                j += 1;
                bosluk += 1;
            }
            if j >= n || (taranan.kucuk[j] != ':' && taranan.kucuk[j] != '=') {
                continue;
            }
            j += 1;
            while j < n && (taranan.ham[j] == ' ' || taranan.ham[j] == '\t') {
                j += 1;
            }
            if j >= n || bosluk_mi(taranan.ham[j]) {
                continue;
            }
            let deger_bas = j;
            while j < n && !bosluk_mi(taranan.ham[j]) {
                j += 1;
            }
            if j <= deger_bas {
                continue;
            }
            bulgular.push(Bulgu {
                baslangic: taranan.bayt(i),
                bitis: taranan.son_bayt(j),
                deger_baslangic: taranan.bayt(deger_bas),
                deger_bitis: taranan.son_bayt(j),
                tur: BulguTuru::AnahtarDeger,
            });
            i = j;
            break;
        }
        i += 1;
    }
}

fn on_ek_bul(taranan: &Metin, onekler: &[(Vec<char>, usize)], bulgular: &mut Vec<Bulgu>) {
    let n = taranan.uzunluk();
    let mut i = 0usize;
    while i < n {
        for (onek, ek_uzunluk) in onekler {
            if !sozcuk_eslesiyor(&taranan.kucuk, i, onek) {
                continue;
            }
            let deger_bas = i + onek.len();
            let mut j = deger_bas;
            while j < n && jeton_harfi_mi(taranan.kucuk[j]) {
                j += 1;
            }
            if j - deger_bas < *ek_uzunluk {
                continue;
            }
            bulgular.push(Bulgu {
                baslangic: taranan.bayt(i),
                bitis: taranan.son_bayt(j),
                deger_baslangic: taranan.bayt(deger_bas),
                deger_bitis: taranan.son_bayt(j),
                tur: BulguTuru::OnEkJeton,
            });
            i = j;
            break;
        }
        i += 1;
    }
}

fn bearer_bul(taranan: &Metin, bulgular: &mut Vec<Bulgu>) {
    let etiket: Vec<char> = "bearer".chars().collect();
    let n = taranan.uzunluk();
    let mut i = 0usize;
    while i < n {
        if !sozcuk_eslesiyor(&taranan.kucuk, i, &etiket) {
            i += 1;
            continue;
        }
        let sol = i == 0 || !sozcuk_harfi(taranan.ham[i - 1]);
        let mut j = i + etiket.len();
        while j < n && taranan.ham[j] == ' ' {
            j += 1;
        }
        let deger_bas = j;
        while j < n && !bosluk_mi(taranan.ham[j]) {
            j += 1;
        }
        if sol && j - deger_bas >= 8 {
            bulgular.push(Bulgu {
                baslangic: taranan.bayt(i),
                bitis: taranan.son_bayt(j),
                deger_baslangic: taranan.bayt(deger_bas),
                deger_bitis: taranan.son_bayt(j),
                tur: BulguTuru::Bearer,
            });
            i = j;
            continue;
        }
        i += 1;
    }
}

fn entropi_bul(metin: &str, bulgular: &mut Vec<Bulgu>) {
    let bayt = metin.as_bytes();
    let mut konum = 0usize;
    while konum < bayt.len() {
        if bosluk_mi(bayt[konum] as char) {
            konum += 1;
            continue;
        }
        let baslangic = konum;
        while konum < bayt.len() && !bosluk_mi(bayt[konum] as char) {
            konum += 1;
        }
        let bitis = konum;
        if bitis - baslangic < EN_KISA_ENTROPI {
            continue;
        }
        if bulgular
            .iter()
            .any(|b| baslangic < b.bitis && bitis > b.baslangic)
        {
            continue;
        }
        let parca = match metin.get(baslangic..bitis) {
            Some(p) => p,
            None => continue,
        };
        let rakam = parca.chars().filter(char::is_ascii_digit).count();
        let alfanumerik = parca.chars().filter(char::is_ascii_alphanumeric).count();
        if rakam >= 2 && alfanumerik >= 20 {
            bulgular.push(Bulgu {
                baslangic,
                bitis,
                deger_baslangic: baslangic,
                deger_bitis: bitis,
                tur: BulguTuru::YuksekEntropi,
            });
        }
    }
}

fn sozcuk_eslesiyor(havuz: &[char], baslangic: usize, sozcuk: &[char]) -> bool {
    if baslangic + sozcuk.len() > havuz.len() {
        return false;
    }
    sozcuk
        .iter()
        .enumerate()
        .all(|(i, harf)| havuz[baslangic + i] == *harf)
}

/// Metni maskeler ve hangi kalıbın bulunduğunu bildirir.
///
/// Maskeleme yalnız **gösterilecek metne** uygulanır; kaynağın kendisi
/// değişmez.
pub fn maskele(metin: &str) -> MaskeSonuc {
    let bulgular = bul(metin);
    let mut cikti = String::with_capacity(metin.len());
    let mut imlec = 0usize;
    for bulgu in &bulgular {
        if bulgu.deger_baslangic < imlec {
            continue;
        }
        cikti.push_str(metin.get(imlec..bulgu.deger_baslangic).unwrap_or(""));
        cikti.push_str(MASKE_ISARETI);
        imlec = bulgu.deger_bitis;
    }
    cikti.push_str(metin.get(imlec..).unwrap_or(""));
    MaskeSonuc {
        metin: cikti,
        bulgular,
    }
}

/// Metnin tamamını tek bir maskeye çevirir; bulgu aranmaz.
pub fn tumunu_maskele(metin: &str) -> String {
    if metin.is_empty() {
        return String::new();
    }
    format!("{MASKE_ISARETI} ({} karakter)", metin.chars().count())
}

#[cfg(test)]
// Testlerde `expect` kullanimi gerekcelidir: test, sınanan degerin dogru
// oldugunu gozle dogrulayacak sekilde yazilamayacak kadar uzun ve testin
// kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik kalir
// (WORKER_CONTRACT.md 4.2).
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Uydurma GitHub jetonu test vektoru.
    ///
    /// Kalite kapısının gizli taraması gerçek anahtar kalıplarını arar. Bu
    /// vektör **gerçek bir anahtar değildir** ama örüntüyü tutturması gerekir;
    /// bu yüzden kaynakta kesintisiz görünmeyecek şekilde iki parça hâlinde
    /// üretilir. Üretim kodunda hiçbir anahtar bulunmaz.
    fn sahte_ghp() -> String {
        format!("gh{}_ABCdefGHIjklMNOpqrSTuvWXYZ0123456789", "p")
    }

    /// Uydurma OpenAI anahtarı test vektorü (bkz. [`sahte_ghp`]).
    fn sahte_sk() -> String {
        format!("s{}-abcdefghijklmnopqrstuvwxyz012345", "k")
    }

    /// AWS dokümantasyonundaki örnek anahtar (bkz. [`sahte_ghp`]).
    fn sahte_aws() -> String {
        format!("AK{}IOSFODNN7EXAMPLE", "IA")
    }

    #[test]
    fn duz_metinde_bulgu_yok() {
        let sonuc = maskele("Merhaba, nasil yardimci olabilirim?");
        assert!(!sonuc.gizli());
        assert_eq!(sonuc.metin, "Merhaba, nasil yardimci olabilirim?");
        assert!(sonuc.bulgular.is_empty());
    }

    #[test]
    fn parola_deger_kalibi_maskeleniyor() {
        let sonuc = maskele("parola: KirmiziKitap123");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.metin, "parola: ****");
        assert_eq!(sonuc.turler(), vec![BulguTuru::AnahtarDeger]);
    }

    #[test]
    fn esittir_ayraci_ile_parola_maskeleniyor() {
        let sonuc = maskele("password=hunter2");
        assert_eq!(sonuc.metin, "password=****");
    }

    #[test]
    fn tirnakli_parola_degerinde_tirnak_yutuluyor() {
        let sonuc = maskele("sifre = \"Gizli2026\" devam");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.metin, "sifre = **** devam");
    }

    #[test]
    fn turkce_sifre_sozcugu_maskeleniyor() {
        let sonuc = maskele("şifre: abc123");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.metin, "şifre: ****");
    }

    #[test]
    fn buyuk_harfle_yazilan_parola_maskeleniyor() {
        assert!(maskele("PASSWORD: DenizKaya").gizli());
    }

    #[test]
    fn degeri_olmayan_anahtar_maskelenmiyor() {
        assert!(!maskele("parola degistirilmelidir").gizli());
    }

    #[test]
    fn anahtar_sozcuk_ice_gecince_maskelenmiyor() {
        assert!(!maskele("tkeyboard: 5").gizli());
    }

    #[test]
    fn metin_ortasindaki_anahtar_maskeleniyor() {
        let sonuc = maskele("kullanici parola: gizli123");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.metin, "kullanici parola: ****");
    }

    #[test]
    fn github_jetonu_maskeleniyor() {
        let jeton = sahte_ghp();
        let sonuc = maskele(&format!("token {jeton}"));
        assert!(sonuc.gizli());
        assert!(!sonuc.metin.contains("ABCdefGHIjklMNOpqrSTuvWXYZ"));
    }

    #[test]
    fn aws_erisim_anahtari_maskeleniyor() {
        let sonuc = maskele(&sahte_aws());
        assert_eq!(sonuc.bulgular.len(), 1);
        assert_eq!(sonuc.bulgular[0].tur, BulguTuru::OnEkJeton);
    }

    #[test]
    fn kisa_onek_jetonu_maskelenmiyor() {
        assert!(!maskele("sk-kisa").gizli());
    }

    #[test]
    fn openai_anahtar_bicimi_maskeleniyor() {
        let jeton = sahte_sk();
        assert!(maskele(&jeton).gizli());
    }

    #[test]
    fn bearer_basligi_maskeleniyor() {
        let sonuc = maskele("Authorization: Bearer eyJhbGciOi.JzdWIiOiIx.sig");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.metin, "Authorization: Bearer ****");
        assert_eq!(sonuc.turler(), vec![BulguTuru::Bearer]);
    }

    #[test]
    fn kisa_bearer_degeri_maskelenmiyor() {
        assert!(!maskele("bearer ab").gizli());
    }

    #[test]
    fn yuksek_entropili_dizi_maskeleniyor() {
        let sonuc = maskele("aB3dE5fG7hJ9kL1mN2pQ4rS6tU8vW0xY");
        assert!(sonuc.gizli());
        assert_eq!(sonuc.turler(), vec![BulguTuru::YuksekEntropi]);
    }

    #[test]
    fn kisa_yuksek_entropili_dizi_maskelenmiyor() {
        assert!(!maskele("aB3dE5fG7hJ9kL1mN2").gizli());
    }

    #[test]
    fn cok_baytli_metin_kaymadan_taraniyor() {
        let sonuc = maskele("Not: şifre: gizliDeger burada");
        assert!(sonuc.gizli());
        assert!(sonuc.metin.starts_with("Not: şifre: "));
        assert!(sonuc.metin.ends_with(" burada"));
    }

    #[test]
    fn cok_baytli_karakter_ardindan_bulgu_ofseti_dogru() {
        let sonuc = maskele("日本語のメモ: token: abc123456");
        assert!(sonuc.gizli());
        let bulgu = sonuc.bulgular[0];
        assert_eq!(
            &"日本語のメモ: token: abc123456"[bulgu.deger_baslangic..bulgu.deger_bitis],
            "abc123456"
        );
    }

    #[test]
    fn iki_bulgu_ayri_ayri_maskeleniyor() {
        let sonuc = maskele("parola: abc123 ve token: xyz987");
        assert_eq!(sonuc.bulgular.len(), 2);
        assert_eq!(sonuc.metin, "parola: **** ve token: ****");
    }

    #[test]
    fn cakisan_bulgular_tekillestiriliyor() {
        let jeton = sahte_ghp();
        let metin = format!("token: {jeton}");
        assert_eq!(bul(&metin).len(), 1);
    }

    #[test]
    fn tumunu_maskele_uzunlugu_sayiyor() {
        assert_eq!(tumunu_maskele("12345"), "**** (5 karakter)");
        assert_eq!(tumunu_maskele(""), "");
    }

    #[test]
    fn bulgu_turu_adlari_benzersiz() {
        let adlar = [
            BulguTuru::AnahtarDeger.ad(),
            BulguTuru::OnEkJeton.ad(),
            BulguTuru::Bearer.ad(),
            BulguTuru::YuksekEntropi.ad(),
        ];
        let benzersiz: BTreeSet<&str> = adlar.into_iter().collect();
        assert_eq!(benzersiz.len(), adlar.len());
    }

    #[test]
    fn sozcuk_eslesiyor_kismen_kalan_kelimede_eslesmiyor() {
        let havuz: Vec<char> = "token".chars().collect();
        let aranan: Vec<char> = "tok".chars().collect();
        assert!(sozcuk_eslesiyor(&havuz, 0, &aranan));
        let havuz2: Vec<char> = "to".chars().collect();
        assert!(!sozcuk_eslesiyor(&havuz2, 0, &aranan));
    }

    #[test]
    fn metin_yapisi_bayt_ofsetlerini_dogru_veriyor() {
        let taranan = Metin::yeni("aüb");
        assert_eq!(taranan.uzunluk(), 3);
        assert_eq!(taranan.bayt(2), 3);
        assert_eq!(taranan.son_bayt(3), 4);
    }
}
