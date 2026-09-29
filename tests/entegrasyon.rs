//! Uçtan uca entegrasyon testleri.
//!
//! Burada tek tek fonksiyonlar değil, **dosya üzerinden gerçek akışlar**
//! sınanır: şablon ekleme, genişletme, pano geçmişi sınırı, gizli mod,
//! dışa/içe aktarma ve istatistik birikimi. Zaman kaynağı testte sabittir
//! (`WORKER_CONTRACT.md` § 5.2).

#[allow(clippy::unwrap_used, clippy::expect_used)]
mod yardimci;

// Entegrasyon testlerinde `expect`/`expect_err` kullanimi gerekcelidir: test,
// sınanan degerin dogru oldugunu gozle dogrulayamayacak kadar uzundur ve
// testin kendisi hata halinde durmalidir. Uretim kodunda bu lint'ler acik
// kalir (WORKER_CONTRACT.md 4.2).
use std::collections::BTreeMap;
use std::time::Duration;

use typefast::aktarma::{self, Bicim};
use typefast::cozumle::{kucult_harf, CozumGerekcesi, Cozumleyici};
use typefast::depo::{self, DepoYolu};
use typefast::genislet::{sabit_an, Ayarlar, Motor, VARSAYILAN_MAKS_DERINLIK};
use typefast::gizli::{self, MASKE_ISARETI};
use typefast::hata::TypeFastHata;
use typefast::kazanc::{self, Istatistik};
use typefast::pano::{PanoGecmisi, PanoKabul, VARSAYILAN_KAPASITE, VARSAYILAN_MAKS_BOYUT};
use typefast::sablon::{Sablon, SablonDeposu};

use yardimci::{iceriyor, GeciciDizin};

/// 2026-09-29T14:05:09Z
const SABIT_AN: u64 = 1_790_690_709;

/// Uydurma GitHub jetonu test vektoru. Kalite kapısının gizli taraması gerçek
/// anahtar kalıplarını arar; bu vektör gerçek bir anahtar değildir ama örüntüyü
/// tutturması gerekir, bu yüzden kaynakta kesintisiz görünmez.
fn sahte_ghp() -> String {
    format!("gh{}_ABCdefGHIjklMNOpqrSTuvWXYZ0123456789", "p")
}

fn motor() -> Motor {
    Motor::zamanli(Ayarlar::isimli("Ceren"), sabit_an(SABIT_AN))
}

fn depo_yolu(dizin: &GeciciDizin) -> DepoYolu {
    DepoYolu::yeni(dizin.yol())
}

fn ornek_depo() -> SablonDeposu {
    let mut depo = SablonDeposu::yeni();
    let mut a = Sablon::yeni(";tesekkur", "Merhaba, teşekkür ederim.");
    a.kategori = "iletisim".to_string();
    let mut b = Sablon::yeni(
        "is parcasi",
        "Talebinizi aldım, en kısa sürede dönüş yapacağım.",
    );
    b.kategori = "destek".to_string();
    let mut c = Sablon::yeni(";kapatis", "İyi çalışmalar dilerim.\n\nSaygılarımla,");
    c.kategori = "iletisim".to_string();
    let mut d = Sablon::yeni("db parola", "parola: KirmiziKitap2026");
    d.gizli = true;
    let mut e = Sablon::yeni("tarihli", "Kayit tarihi: {{tarih}} / {{saat}}");
    e.kategori = "kayit".to_string();
    depo.ekle(a).expect("a eklenmeli");
    depo.ekle(b).expect("b eklenmeli");
    depo.ekle(c).expect("c eklenmeli");
    depo.ekle(d).expect("d eklenmeli");
    depo.ekle(e).expect("e eklenmeli");
    depo
}

#[test]
fn sablon_ekle_genislet_dosya_gidis_donusu() {
    let dizin = GeciciDizin::yeni("ekle-genislet").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);

    let mut depo = depo::sablonlari_yukle(&yol).expect("bos depo okunmali");
    assert!(depo.bos());
    depo.ekle(Sablon::yeni(";tesekkur", "Merhaba, teşekkür ederim."))
        .expect("eklenmeli");
    depo::sablonlari_kaydet(&yol, &depo).expect("kaydedilmeli");

    let geri = depo::sablonlari_yukle(&yol).expect("depo okunmali");
    let sablon = geri.bul(";tesekkur").expect("sablon bulunmali");
    let genisletme = motor().genislet(&geri, sablon).expect("genisletilmeli");
    assert_eq!(genisletme.metin, "Merhaba, teşekkür ederim.");
    assert_eq!(genisletme.adim, 0);
}

#[test]
fn yerlesik_tarih_saat_genislemede_kullaniliyor() {
    let dizin = GeciciDizin::yeni("yerlesik").expect("dizin olusmali");
    assert!(dizin.yol().exists());
    let depo = ornek_depo();
    let sablon = depo.bul("tarihli").expect("sablon olmali");
    let g = motor().genislet(&depo, sablon).expect("genisletilmeli");
    assert_eq!(g.metin, "Kayit tarihi: 2026-09-29 / 14:05:09");
}

#[test]
fn kullanici_degeri_tanimli_yer_tutucuyu_eziyor() {
    let depo = ornek_depo();
    let degerler = BTreeMap::from([("musteri".to_string(), "Ayşe Yılmaz".to_string())]);
    let g = motor()
        .genislet_metin(&depo, "Sayın {{musteri|Kıymetli Müşteri}},", &degerler)
        .expect("genisletilmeli");
    assert_eq!(g.metin, "Sayın Ayşe Yılmaz,");
}

#[test]
fn cozumleyici_metindeki_en_uzun_kisayolu_seciyor() {
    let mut depo = ornek_depo();
    let kisa = Sablon::yeni("is", "Merhaba.");
    depo.ekle(kisa).expect("eklenmeli");
    let cozumleyici = Cozumleyici::yeni(&depo);
    let cozum = cozumleyici
        .coz("Merhaba, is parcasi olsun.")
        .expect("cozulmeli");
    assert_eq!(cozum.tetik, "is parcasi");
    assert_eq!(cozum.gerekce, CozumGerekcesi::EnUzun);
    assert_eq!(cozum.baslangic, 9);
}

#[test]
fn cozumleyici_esit_kullanimda_belirsizlik_donduruyor() {
    let mut depo = SablonDeposu::yeni();
    depo.ekle(Sablon::yeni(";is", "biri")).expect("eklenmeli");
    depo.ekle(Sablon::yeni("is", "biri")).expect("eklenmeli");
    let cozumleyici = Cozumleyici::yeni(&depo);
    match cozumleyici.coz("Lütfen ;is") {
        Err(TypeFastHata::CozumBelirsiz { adaylar, .. }) => assert_eq!(adaylar.len(), 2),
        diger => panic!("belirsizlik bekleniyordu: {diger:?}"),
    }
}

#[test]
fn ayni_tetik_iki_kez_eklenemiyor() {
    let mut depo = SablonDeposu::yeni();
    depo.ekle(Sablon::yeni(";tesekkur", "bir"))
        .expect("eklenmeli");
    let hata = depo
        .ekle(Sablon::yeni(";tesekkur", "iki"))
        .expect_err("cakisma bekleniyor");
    assert!(matches!(hata, TypeFastHata::TetikCakismasi { .. }));
}

#[test]
fn pano_halkasi_kapasiteyi_asmaz() {
    let dizin = GeciciDizin::yeni("pano-halka").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let mut pano = PanoGecmisi::yeni(3, VARSAYILAN_MAKS_BOYUT);
    for i in 0..5 {
        assert!(matches!(
            pano.ekle(&format!("kayit-{i}"), i),
            PanoKabul::Eklendi(_)
        ));
    }
    depo::panoyu_kaydet(&yol, &pano).expect("kaydedilmeli");
    let geri = depo::panoyu_yukle(&yol).expect("okunmali");
    assert_eq!(geri.uzunluk(), 3);
    assert_eq!(geri.listele()[0].metin, "kayit-4");
    assert_eq!(geri.listele()[2].metin, "kayit-2");
}

#[test]
fn pano_boyut_siniri_asilan_kaydi_atlayip_sayaci_artirir() {
    let mut pano = PanoGecmisi::yeni(VARSAYILAN_KAPASITE, 32);
    let uzun = "x".repeat(33);
    assert_eq!(pano.ekle(&uzun, 0), PanoKabul::Atlandi);
    assert_eq!(pano.atlanan, 1);
    assert!(pano.bos());
}

#[test]
fn gizli_kayit_listede_maskeli_gorunur_ozgun_metin_donulur() {
    let dizin = GeciciDizin::yeni("gizli-kayit").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let mut pano = PanoGecmisi::yeni(VARSAYILAN_KAPASITE, VARSAYILAN_MAKS_BOYUT);
    pano.ekle("parola: KirmiziKitap2026", 1);
    pano.ekle("Rapor metni", 2);
    depo::panoyu_kaydet(&yol, &pano).expect("kaydedilmeli");

    let geri = depo::panoyu_yukle(&yol).expect("okunmali");
    let gizli_kayit = geri
        .gizli_kayitlar()
        .first()
        .copied()
        .expect("gizli kayit olmali");
    assert!(gizli_kayit.gizli);
    assert!(!iceriyor(&gizli_kayit.gorunen_metin(), "KirmiziKitap2026"));
    assert!(iceriyor(&gizli_kayit.gorunen_metin(), MASKE_ISARETI));
    assert_eq!(gizli_kayit.metin, "parola: KirmiziKitap2026");
    assert!(geri.ara("KirmiziKitap2026", false).is_empty());
    assert_eq!(geri.ara("KirmiziKitap2026", true).len(), 1);
}

#[test]
fn gizli_sablon_genislemesi_maskeli_yazilir() {
    let depo = ornek_depo();
    let sablon = depo.bul("db parola").expect("gizli sablon olmali");
    assert!(sablon.gizli);
    let g = motor().genislet(&depo, sablon).expect("genisletilmeli");
    assert_eq!(g.metin, "parola: KirmiziKitap2026");
    let gorunen = gizli::tumunu_maskele(&g.metin);
    assert!(!iceriyor(&gorunen, "KirmiziKitap2026"));
}

#[test]
fn mask_komutu_parola_token_ve_anahtar_kaliplarini_bulur() {
    let ornekler = [
        ("parola: KirmiziKitap2026", "anahtar-deger"),
        ("api_key=abc123def456", "anahtar-deger"),
        (&sahte_ghp(), "onek-jeton"),
        (
            "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.abc.def",
            "bearer",
        ),
        ("aB3dE5fG7hJ9kL1mN2pQ4rS6tU8vW0xY", "yuksek-entropi"),
    ];
    for (metin, beklenen) in ornekler {
        let sonuc = gizli::maskele(metin);
        let turler = sonuc.turler();
        let adlar: Vec<&str> = turler.iter().map(|t| t.ad()).collect();
        assert!(
            adlar.contains(&beklenen),
            "'{metin}' icin {beklenen} bekleniyordu, bulunan: {adlar:?}"
        );
    }
}

#[test]
fn duz_metinde_yanlis_pozitif_uretilmiyor() {
    let ornekler = [
        "Merhaba, size nasil yardimci olabilirim?",
        "Toplanti 14:30'da baslayacak.",
        "Fatura no: 2026-4471",
        "Monitör 27 inç, 2560x1440",
    ];
    for metin in ornekler {
        let sonuc = gizli::maskele(metin);
        assert!(sonuc.bulgular.is_empty(), "'{metin}' yanlis pozitif verdi");
        assert_eq!(sonuc.metin, metin);
    }
}

#[test]
fn json_disa_ice_aktarma_gidis_donusu() {
    let depo = ornek_depo();
    let metin = aktarma::json_paket(&depo).expect("paket uretilmeli");
    let geri = aktarma::json_oku(&metin).expect("paket okunmali");
    assert_eq!(depo.uzunluk(), geri.uzunluk());
    assert_eq!(
        geri.bul(";tesekkur").map(|s| s.govde.as_str()),
        Some("Merhaba, teşekkür ederim.")
    );
    assert_eq!(geri.bul("db parola").map(|s| s.gizli), Some(true));
}

#[test]
fn metin_disa_ice_aktarma_gidis_donusu() {
    let depo = ornek_depo();
    let metin = aktarma::metin_liste(&depo);
    assert!(metin.starts_with(aktarma::BASLIK));
    let geri = aktarma::metin_oku(&metin).expect("liste okunmali");
    assert_eq!(depo.uzunluk(), geri.uzunluk());
    let kapatis = geri.bul(";kapatis").expect("sablon olmali");
    assert!(iceriyor(&kapatis.govde, "\n"));
}

#[test]
fn aktarma_pano_gecmisi_ve_kullanim_sayacini_aktarmaz() {
    let depo = ornek_depo();
    let json = aktarma::json_paket(&depo).expect("paket uretilmeli");
    assert!(!iceriyor(&json, "kayitlar"));
    let metin = aktarma::metin_liste(&depo);
    assert!(!iceriyor(&metin, "pano"));
}

#[test]
fn bozuk_depo_dosyasi_hata_donduruyor() {
    let dizin = GeciciDizin::yeni("bozuk-depo").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    std::fs::write(yol.sablon_dosyasi(), "{\"sablonlar\": [ }").expect("yazilmali");
    match depo::sablonlari_yukle(&yol) {
        Err(TypeFastHata::BozukDepo { ayrinti, .. }) => assert!(!ayrinti.is_empty()),
        diger => panic!("bozuk depo hatasi bekleniyordu: {diger:?}"),
    }
}

#[test]
fn atomik_yazma_gecici_dosya_birakmiyor_ve_guncelliyor() {
    let dizin = GeciciDizin::yeni("atomik-it").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let mut depo = ornek_depo();
    depo::sablonlari_kaydet(&yol, &depo).expect("kaydedilmeli");
    assert!(!depo::gecici_yol(&yol.sablon_dosyasi()).exists());

    let ilk = std::fs::read_to_string(yol.sablon_dosyasi()).expect("okunmali");
    depo.ekle(Sablon::yeni("yeni", "yeni govde"))
        .expect("eklenmeli");
    depo::sablonlari_kaydet(&yol, &depo).expect("guncellenmeli");
    let ikinci = std::fs::read_to_string(yol.sablon_dosyasi()).expect("okunmali");
    assert_eq!(ikinci.matches("\"tetik\"").count(), depo.uzunluk());
    assert!(ikinci.len() > ilk.len());
}

#[test]
fn bozuk_metin_listesi_hata_donduruyor() {
    let metin = format!("{}\na\tb\tc", aktarma::BASLIK);
    assert!(aktarma::metin_oku(&metin).is_err());
    assert!(aktarma::json_oku("{ bozuk").is_err());
}

#[test]
fn bicim_sezgisi_dosya_iceriginden_calisiyor() {
    let depo = ornek_depo();
    assert_eq!(
        aktarma::bicim_sezgile(&aktarma::json_paket(&depo).expect("paket")),
        Bicim::Json
    );
    assert_eq!(
        aktarma::bicim_sezgile(&aktarma::metin_liste(&depo)),
        Bicim::Metin
    );
}

#[test]
fn kazanc_olcumu_tus_sayacina_dayaniyor() {
    let k = kazanc::olc(
        "Merhaba, teşekkür ederim.",
        ";t",
        Duration::from_millis(2),
        1.0,
    );
    assert_eq!(k.gerekli_tus, 25);
    assert_eq!(k.gercek_tus, 3);
    assert_eq!(k.kazanc, 22);
    assert!(!k.zararli());
    assert_eq!(k.sure_mikrosaniye(), 2000);
}

#[test]
fn kazanc_olculmedigi_zaman_istatistik_bos_kalir() {
    let dizin = GeciciDizin::yeni("olculmedi").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let i = depo::istatistigi_yukle(&yol).expect("okunmali");
    assert!(i.olculmedi());
    assert_eq!(i.ortalama_sn(), None);
    assert_eq!(i.genisletme, 0);
}

#[test]
fn istatistik_birikimi_dosyaya_yazilip_okunuyor() {
    let dizin = GeciciDizin::yeni("istatistik").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let mut i = Istatistik::yeni();
    for _ in 0..3 {
        let k = kazanc::olc("abcdefghijklmnop", ";uzun", Duration::from_millis(1), 1.0);
        i.ekle(";uzun", k, 1.0);
    }
    depo::istatistigi_kaydet(&yol, &i).expect("kaydedilmeli");
    let geri = depo::istatistigi_yukle(&yol).expect("okunmali");
    assert_eq!(geri.genisletme, 3);
    assert_eq!(geri.tetikler[";uzun"].adet, 3);
    assert_eq!(geri.kazanilan_tus, 30);
    assert!(geri.ortalama_sn().unwrap_or(0.0) > 0.0);
}

#[test]
fn turkce_ve_unicode_tetikler_calisiyor() {
    let mut depo = SablonDeposu::yeni();
    depo.ekle(Sablon::yeni(
        ";görüşürüz",
        "Görüşmek üzere, iyi çalışmalar.",
    ))
    .expect("eklenmeli");
    depo.ekle(Sablon::yeni("şey", "bu bir şey"))
        .expect("eklenmali");
    let cozumleyici = Cozumleyici::yeni(&depo);
    assert_eq!(
        cozumleyici.coz(";Görüşürüz!").expect("cozulmeli").tetik,
        ";görüşürüz"
    );
    assert_eq!(
        cozumleyici.coz("Bu bir şey mi?").expect("cozulmeli").tetik,
        "şey"
    );
}

#[test]
fn turkce_kucult_harf_indirgemesi() {
    assert_eq!(kucult_harf("IŞIK"), "işik");
    assert_eq!(kucult_harf("İSTANBUL"), "istanbul");
    assert_eq!(kucult_harf("Çalışıyor"), "çalişiyor");
}

#[test]
fn ic_ice_cagri_ve_dongu_koruması_uygulama_hizasinda() {
    let mut depo = SablonDeposu::yeni();
    depo.ekle(Sablon::yeni("a", "{{>b}}")).expect("eklenmeli");
    depo.ekle(Sablon::yeni("b", "{{>a}}")).expect("eklenmeli");
    let hata = motor()
        .genislet_metin(&depo, "{{>a}}", &BTreeMap::new())
        .expect_err("cevrim hatasi bekleniyor");
    assert!(matches!(hata, TypeFastHata::Cevrim { .. }));
}

#[test]
fn derinlik_siniri_varsayilan_degerde_sekiz() {
    assert_eq!(VARSAYILAN_MAKS_DERINLIK, 8);
    let mut depo = SablonDeposu::yeni();
    for i in 0..12 {
        depo.ekle(Sablon::yeni(
            &format!("d{i}"),
            &format!("{{{{>d{}}}}} son", i + 1),
        ))
        .expect("eklenmeli");
    }
    depo.ekle(Sablon::yeni("d12", "son")).expect("eklenmeli");
    match motor().genislet_metin(&depo, "{{>d0}}", &BTreeMap::new()) {
        Err(TypeFastHata::DerinlikSiniri { sinir }) => assert_eq!(sinir, 8),
        diger => panic!("derinlik hatasi bekleniyordu: {diger:?}"),
    }
}

#[test]
fn pano_arama_harf_duyarsiz_calisiyor() {
    let mut pano = PanoGecmisi::yeni(VARSAYILAN_KAPASITE, VARSAYILAN_MAKS_BOYUT);
    pano.ekle("Fatura No 2026-4471", 1);
    pano.ekle("Toplanti notu", 2);
    assert_eq!(pano.ara("fatura", false).len(), 1);
    assert_eq!(pano.ara("TOPLANTI", false).len(), 1);
    assert_eq!(pano.ara("bulunmayan", false).len(), 0);
}

#[test]
fn depo_dosyalari_ayri_ayri_silinip_yuklenebiliyor() {
    let dizin = GeciciDizin::yeni("dosya-ayrimi").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    depo::sablonlari_kaydet(&yol, &ornek_depo()).expect("sablonlar yazilmali");
    let mut pano = PanoGecmisi::yeni(5, 100);
    pano.ekle("bir kayit", 1);
    depo::panoyu_kaydet(&yol, &pano).expect("pano yazilmali");
    let mut i = Istatistik::yeni();
    i.ekle(
        ";tesekkur",
        kazanc::olc("abcd", ";t", Duration::from_millis(1), 1.0),
        1.0,
    );
    depo::istatistigi_kaydet(&yol, &i).expect("istatistik yazilmali");

    std::fs::remove_file(yol.pano_dosyasi()).expect("pano silinmeli");
    assert!(
        depo::sablonlari_yukle(&yol)
            .expect("sablonlar okunmali")
            .uzunluk()
            > 0
    );
    assert!(depo::panoyu_yukle(&yol).expect("pano okunmali").bos());
    assert!(!depo::istatistigi_yukle(&yol)
        .expect("istatistik okunmali")
        .olculmedi());
}

#[test]
fn tum_eslesmeler_cakismadan_diziliyor() {
    let depo = ornek_depo();
    let cozumleyici = Cozumleyici::yeni(&depo);
    let liste = cozumleyici.tum_eslesmeler(";tesekkur ve is parcasi");
    let adlar: Vec<&str> = liste.iter().map(|c| c.tetik.as_str()).collect();
    assert_eq!(adlar, vec![";tesekkur", "is parcasi"]);
    for pencere in liste.windows(2) {
        assert!(pencere[0].bitis <= pencere[1].baslangic);
    }
}

#[test]
fn disa_aktarma_dosyaya_yazilip_ice_aktariliyor() {
    let dizin = GeciciDizin::yeni("aktarma-dosya").expect("dizin olusmali");
    let kaynak = ornek_depo();
    let hedef = dizin.yol().join("paket.json");
    let icerik = aktarma::disa_aktar(&kaynak, Bicim::Json).expect("disa aktarilmali");
    aktarma::dosyaya_yaz(&hedef, &icerik).expect("yazilmali");
    let okunan = std::fs::read_to_string(&hedef).expect("okunmali");
    let alinan =
        aktarma::ice_aktar(&okunan, aktarma::bicim_sezgile(&okunan)).expect("ice aktarilmali");
    assert_eq!(alinan.uzunluk(), kaynak.uzunluk());

    let metin_hedef = dizin.yol().join("liste.txt");
    let liste = aktarma::disa_aktar(&kaynak, Bicim::Metin).expect("disa aktarilmali");
    aktarma::dosyaya_yaz(&metin_hedef, &liste).expect("yazilmali");
    let metin_okunan = std::fs::read_to_string(&metin_hedef).expect("okunmali");
    let metin_alinan = aktarma::metin_oku(&metin_okunan).expect("liste okunmali");
    assert_eq!(metin_alinan.uzunluk(), kaynak.uzunluk());
}

#[test]
fn kullanim_sayaci_ve_istatistik_birlikte_birikir() {
    let dizin = GeciciDizin::yeni("birikim").expect("dizin olusmali");
    let yol = depo_yolu(&dizin);
    let mut depo = ornek_depo();
    let sablon = depo.bul(";tesekkur").cloned().expect("sablon olmali");
    let mut pano = PanoGecmisi::yeni(10, 1000);
    let mut i = Istatistik::yeni();

    for _ in 0..2 {
        let g = motor().genislet(&depo, &sablon).expect("genisletilmeli");
        pano.ekle(&g.metin, 0);
        let k = kazanc::olc(&g.metin, ";tesekkur", Duration::from_millis(1), 1.0);
        i.ekle(";tesekkur", k, 1.0);
        depo.kullanim_artir(";tesekkur").expect("sayac artmali");
    }
    depo::sablonlari_kaydet(&yol, &depo).expect("sablonlar yazilmali");
    depo::panoyu_kaydet(&yol, &pano).expect("pano yazilmali");
    depo::istatistigi_kaydet(&yol, &i).expect("istatistik yazilmali");

    let s = depo::sablonlari_yukle(&yol).expect("okunmali");
    assert_eq!(s.bul(";tesekkur").map(|x| x.kullanim), Some(2));
    assert_eq!(depo::panoyu_yukle(&yol).expect("okunmali").uzunluk(), 1);
    assert_eq!(
        depo::istatistigi_yukle(&yol).expect("okunmali").genisletme,
        2
    );
}

#[test]
fn ornek_sablon_koleksiyonu_yuklenebiliyor() {
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ornek");
    let yol = DepoYolu::yeni(kok);
    let depo = depo::sablonlari_yukle(&yol).expect("ornek deposu okunmali");
    assert!(
        depo.uzunluk() >= 15,
        "ornek koleksiyonda >=15 sablon olmali"
    );
    let cozumleyici = Cozumleyici::yeni(&depo);
    assert!(cozumleyici.coz(";tesekkur").is_ok());
    // Ornek koleksiyondaki her sablon genisletilebilir olmali.
    for sablon in depo.tumune() {
        let sonuc = motor().genislet(&depo, sablon);
        assert!(sonuc.is_ok(), "'{}' genisletilemedi", sablon.tetik);
    }
    // Ornek koleksiyon disari aktarilip geri alinabilir olmali.
    let paket = aktarma::disa_aktar(&depo, Bicim::Json).expect("disa aktarilmali");
    let geri = aktarma::ice_aktar(&paket, Bicim::Json).expect("ice aktarilmali");
    assert_eq!(geri.uzunluk(), depo.uzunluk());
    let liste = aktarma::disa_aktar(&depo, Bicim::Metin).expect("liste uretilmeli");
    let geri_metin = aktarma::ice_aktar(&liste, Bicim::Metin).expect("liste okunmali");
    assert_eq!(geri_metin.uzunluk(), depo.uzunluk());
}
