//! Komut satırı arayüzü (`clap` ile türetilmiş).
//!
//! Alt komutlar: `add`, `list`, `expand`, `clipboard`, `search`, `export`,
//! `import`, `stats`, `mask`, `conflicts`.
//!
//! Yol çözümlemesi (rapor `b09`):
//! 1. `--depo` komut satırı argümanı
//! 2. `TYPEFAST_DEPO` ortam değişkeni
//! 3. Çalıştırılabilir dosyanın yanındaki dizin
//!
//! `--oturum` bayrağı depoyu yalnız bellekte tutar ve **hiçbir şeyi diske
//! yazmaz**; hassas veriyle denemek için kullanılır.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};

use crate::aktarma::{self, Bicim};
use crate::cozumle::Cozumleyici;
use crate::depo::{self, DepoYolu};
use crate::genislet::{Ayarlar, Motor};
use crate::gizli;
use crate::hata::{Sonuc, TypeFastHata};
use crate::kazanc::{Kazanc, VARSAYILAN_TUS_CARPANI};
use crate::pano::PanoKabul;
use crate::sablon::Sablon;

/// TypeFast — genel metin genişletici ve pano geçmişi.
#[derive(Debug, Parser)]
#[command(
    name = "typefast",
    version,
    about = "Genel metin icin ongorulebilir sablon genisletici, kisayol cozumleyici ve duz metin pano gecmisi.",
    long_about = None
)]
pub struct KomutSatiri {
    /// Depo dosyalarinin bulunacagi dizin.
    #[arg(long, value_name = "YOL", global = true)]
    pub depo: Option<PathBuf>,

    /// Yalnizca bellekte calis; hicbir seyi diske yazma.
    #[arg(long, global = true)]
    pub oturum: bool,

    /// `{{isim}}` yerlesiginin degeri.
    #[arg(long, value_name = "AD", global = true)]
    pub isim: Option<String>,

    /// Yapilacak islem.
    #[command(subcommand)]
    pub komut: AltKomut,
}

/// Alt komutlar.
#[derive(Debug, Subcommand)]
pub enum AltKomut {
    /// Yeni sablon ekler.
    Add(Ekle),
    /// Sablonlari listeler.
    List(Liste),
    /// Bir sablonu genisletir.
    Expand(Genislet),
    /// Pano gecmisi islemleri.
    #[command(subcommand)]
    Clipboard(PanoKomutu),
    /// Sablon ve pano gecmisinde arama yapar.
    Search(Ara),
    /// Sablonlari disa aktarir.
    Export(Aktar),
    /// Sablonlari ice aktarir.
    Import(Al),
    /// Kazanc istatistigini gosterir.
    Stats(Stats),
    /// Metindeki sirlari maskeler.
    Mask(Maskele),
    /// Ayni kelime dizisini paylasan tetikleri gosterir.
    Conflicts(Cakisma),
    /// Depo dosyalarinin yollarini gosterir.
    Yollar,
}

/// `add` argümanları.
#[derive(Debug, Args)]
pub struct Ekle {
    /// Kisayol tetigi.
    pub tetik: String,
    /// Genisletilecek metin.
    pub govde: String,
    /// Kategori.
    #[arg(long, value_name = "AD")]
    pub kategori: Option<String>,
    /// Kisa aciklama.
    #[arg(long, value_name = "METIN")]
    pub aciklama: Option<String>,
    /// Genisletme ciktisini maskele ve pano gecmisine yazma.
    #[arg(long)]
    pub gizli: bool,
    /// Genislemede kullanilacak yer tutucu degerleri (`anahtar=deger`).
    #[arg(long = "deger", value_name = "ANAHTAR=DEGER")]
    pub degerler: Vec<String>,
}

/// `list` argümanları.
#[derive(Debug, Args)]
pub struct Liste {
    /// Yalnizca bu kategori.
    #[arg(long, value_name = "AD")]
    pub kategori: Option<String>,
    /// En cok kullanilan N sablonu goster.
    #[arg(long = "en-cok", value_name = "N")]
    pub en_cok: Option<usize>,
    /// Govde metnini de yazdir.
    #[arg(long)]
    pub ayrinti: bool,
}

/// `expand` argümanları.
#[derive(Debug, Args)]
pub struct Genislet {
    /// Kisayol tetigi.
    pub tetik: String,
    /// Yer tutucu degerleri (`anahtar=deger`).
    #[arg(long = "deger", value_name = "ANAHTAR=DEGER")]
    pub degerler: Vec<String>,
    /// Hangi olcutle cozuldugunu yazdir.
    #[arg(long)]
    pub gerekce: bool,
}

/// Pano alt komutları.
#[derive(Debug, Subcommand)]
pub enum PanoKomutu {
    /// Panoya metin ekler.
    #[command(name = "ekle")]
    Ekle {
        /// Eklenecek metin.
        metin: String,
    },
    /// Panodan metin okur (pano dinleme yoktur).
    #[command(name = "oku")]
    Oku {
        /// Kayit numarasi.
        id: u64,
        /// Maskeyi kaldirarak ozgun metni yazdir.
        #[arg(long)]
        coz: bool,
    },
    /// Gecmisi listeler.
    #[command(name = "liste")]
    Liste {
        /// En fazla kayit.
        #[arg(long, default_value_t = 20)]
        adet: usize,
        /// Gizli kayitlari da goster.
        #[arg(long)]
        gizliler: bool,
    },
    /// Gecmisi arar.
    #[command(name = "ara")]
    Ara {
        /// Arama sorgusu.
        sorgu: String,
    },
    /// Gecmisi siler.
    #[command(name = "temizle")]
    Temizle,
}

/// `search` argümanları.
#[derive(Debug, Args)]
pub struct Ara {
    /// Arama sorgusu.
    pub sorgu: String,
    /// Pano gecmisinde de ara.
    #[arg(long)]
    pano: bool,
}

/// `export` argümanları.
#[derive(Debug, Args)]
pub struct Aktar {
    /// Hedef dosya; verilmezse standart cikti.
    #[arg(value_name = "DOSYA")]
    pub dosya: Option<PathBuf>,
    /// Biçim: `json` veya `metin`.
    #[arg(long, default_value = "json")]
    pub bicim: String,
}

/// `import` argümanları.
#[derive(Debug, Args)]
pub struct Al {
    /// Kaynak dosya; verilmezse standart girdi okunur.
    #[arg(value_name = "DOSYA")]
    pub dosya: Option<PathBuf>,
    /// Biçim: `json`, `metin` veya `otomatik`.
    #[arg(long, default_value = "otomatik")]
    pub bicim: String,
    /// Var olanlari degistir.
    #[arg(long)]
    pub degistir: bool,
}

/// `stats` argümanları.
#[derive(Debug, Args)]
pub struct Stats {
    /// En cok kazandiran N tetigi goster.
    #[arg(long = "en-cok", value_name = "N", default_value_t = 5)]
    pub en_cok: usize,
    /// Bir karakterin kac tus vurusu sayildigi.
    #[arg(long = "tus-carpani", value_name = "KATSAYI", default_value_t = VARSAYILAN_TUS_CARPANI)]
    pub tus_carpani: f64,
}

/// `mask` argümanları.
#[derive(Debug, Args)]
pub struct Maskele {
    /// Maskelenecek metin.
    pub metin: String,
    /// Yalnizca bulgu turunu yazdir.
    #[arg(long)]
    pub tur: bool,
}

/// `conflicts` argümanları.
#[derive(Debug, Args)]
pub struct Cakisma {
    /// Cikti bicimi: `metin` veya `json`.
    #[arg(long, default_value = "metin")]
    pub bicim: String,
}

/// Programın çalıştığı bağlam: depo yolu, oturum kipi ve motor.
fn cikti() -> Box<dyn Write> {
    Box::new(std::io::stdout())
}

fn satir_yaz(satir: &str) -> Sonuc<()> {
    let mut c = cikti();
    writeln!(c, "{satir}").map_err(|hata| TypeFastHata::YazmaHatasi {
        yol: PathBuf::from("<stdout>"),
        hata,
    })
}

/// Programin calistigi baglam: depo yolu, oturum kipi ve motor.
pub struct Baglam {
    /// Kullanılacak depo dizini.
    pub yol: DepoYolu,
    /// Oturum kipi etkin mi?
    pub oturum: bool,
    /// Genişletme motoru.
    pub motor: Motor,
}

/// Depo dizinini çözer ve motoru kurar.
pub fn baglam_kur(komut: &KomutSatiri) -> Sonuc<Baglam> {
    let yol = match &komut.depo {
        Some(yol) => DepoYolu::yeni(yol.clone()),
        None => DepoYolu::yeni(varsayilan_kok()),
    };
    let ayarlar = Ayarlar::isimli(komut.isim.as_deref().unwrap_or("Kullanici"));
    Ok(Baglam {
        yol,
        oturum: komut.oturum,
        motor: Motor::yeni(ayarlar),
    })
}

/// Depo dizinini çözer: `--depo`, `TYPEFAST_DEPO`, çalıştırılabilirin yanı.
pub fn varsayilan_kok() -> PathBuf {
    if let Some(ortam) = std::env::var_os("TYPEFAST_DEPO") {
        if !ortam.is_empty() {
            return PathBuf::from(ortam);
        }
    }
    if let Ok(yol) = std::env::current_exe() {
        if let Some(ust) = yol.parent() {
            return ust.to_path_buf();
        }
    }
    PathBuf::from(".")
}

/// Komutu çalıştırır ve standart çıktıya yazar.
pub fn calistir(komut: &KomutSatiri) -> Sonuc<i32> {
    let baglam = baglam_kur(komut)?;
    match &komut.komut {
        AltKomut::Add(args) => add_calistir(&baglam, args),
        AltKomut::List(args) => liste_calistir(&baglam, args),
        AltKomut::Expand(args) => genislet_calistir(&baglam, args),
        AltKomut::Clipboard(pano) => pano_calistir(&baglam, pano),
        AltKomut::Search(args) => ara_calistir(&baglam, args),
        AltKomut::Export(args) => export_calistir(&baglam, args),
        AltKomut::Import(args) => import_calistir(&baglam, args),
        AltKomut::Stats(args) => stats_calistir(&baglam, args),
        AltKomut::Mask(args) => mask_calistir(args),
        AltKomut::Conflicts(args) => conflicts_calistir(&baglam, args),
        AltKomut::Yollar => yollar_calistir(&baglam),
    }
}

/// `--oturum` kipinde diske yazmayı engeller.
fn kaydet_sablonlar(baglam: &Baglam, depo: &crate::sablon::SablonDeposu) -> Sonuc<()> {
    if baglam.oturum {
        return Ok(());
    }
    depo::sablonlari_kaydet(&baglam.yol, depo)
}

fn kaydet_pano(baglam: &Baglam, pano: &crate::pano::PanoGecmisi) -> Sonuc<()> {
    if baglam.oturum {
        return Ok(());
    }
    depo::panoyu_kaydet(&baglam.yol, pano)
}

fn kaydet_istatistik(baglam: &Baglam, i: &crate::kazanc::Istatistik) -> Sonuc<()> {
    if baglam.oturum {
        return Ok(());
    }
    depo::istatistigi_kaydet(&baglam.yol, i)
}

fn degerleri_coz(args: &[String]) -> Sonuc<BTreeMap<String, String>> {
    let mut harita = BTreeMap::new();
    for girdi in args {
        let (anahtar, deger) =
            girdi
                .split_once('=')
                .ok_or_else(|| TypeFastHata::AktarmaHatasi {
                    yol: None,
                    ayrinti: format!("'{girdi}' 'anahtar=deger' biciminde olmali"),
                })?;
        harita.insert(anahtar.trim().to_string(), deger.to_string());
    }
    Ok(harita)
}

fn add_calistir(baglam: &Baglam, args: &Ekle) -> Sonuc<i32> {
    let mut depo = depo::sablonlari_yukle(&baglam.yol)?;
    let mut sablon = Sablon::yeni(&args.tetik, &args.govde);
    sablon.kategori = args.kategori.clone().unwrap_or_default();
    sablon.aciklama = args.aciklama.clone().unwrap_or_default();
    sablon.gizli = args.gizli || !gizli::bul(&args.govde).is_empty();
    let degerler = degerleri_coz(&args.degerler)?;

    // Govde bir kez genisletilerek **dogrulanir** ama oldugu gibi saklanir:
    // `{{isim}}` gibi yer tutucular `expand` aninda, o anki degerle cozulur.
    let motor = &baglam.motor;
    let genisletme = motor.genislet_metin(&depo, &sablon.govde, &degerler)?;

    if sablon.gizli {
        let gizli_sonuc = gizli::maskele(&sablon.govde);
        if !gizli_sonuc.gizli() {
            satir_yaz("uyari: --gizli verildi ama metinde parola/token kalibi bulunamadi")?;
        }
    }

    depo.ekle(sablon.clone())?;
    kaydet_sablonlar(baglam, &depo)?;
    satir_yaz(&format!(
        "eklendi: {} ({} karakter, {} adim dogrulandi)",
        sablon.tetik,
        sablon.govde.chars().count(),
        genisletme.adim
    ))?;
    Ok(0)
}

fn liste_calistir(baglam: &Baglam, args: &Liste) -> Sonuc<i32> {
    let depo = depo::sablonlari_yukle(&baglam.yol)?;
    if depo.bos() {
        satir_yaz("depo bos; eklemek icin: typefast add <tetik> <govde>")?;
        return Ok(0);
    }
    let secili = match args.en_cok {
        Some(adet) => depo.en_cok_kullanilan(adet),
        None => match &args.kategori {
            Some(k) => depo.kategoriye_gore(k),
            None => depo.tumune().iter().collect(),
        },
    };
    satir_yaz(&format!(
        "{} sablon{}",
        secili.len(),
        match &args.kategori {
            Some(k) if args.en_cok.is_none() => format!(" / {k}"),
            _ => String::new(),
        }
    ))?;
    for sablon in secili {
        let ozet = if args.ayrinti {
            sablon.govde.clone()
        } else {
            sablon.ozet()
        };
        satir_yaz(&format!(
            "{:<16} {:>6}  {}{}{}",
            sablon.tetik,
            sablon.kullanim,
            if sablon.gizli { "[gizli] " } else { "" },
            if sablon.kategori.is_empty() {
                String::new()
            } else {
                format!("({}) ", sablon.kategori)
            },
            ozet
        ))?;
    }
    Ok(0)
}

fn genislet_calistir(baglam: &Baglam, args: &Genislet) -> Sonuc<i32> {
    let depo = depo::sablonlari_yukle(&baglam.yol)?;
    let degerler = degerleri_coz(&args.degerler)?;

    // Once metin olarak cozumle: kullanici metin icinde yazmis olabilir.
    let cozumleyici = Cozumleyici::yeni(&depo);
    let cozum = cozumleyici.coz(&args.tetik)?;
    let sablon = depo
        .bul(&cozum.tetik)
        .cloned()
        .ok_or_else(|| TypeFastHata::BilinmeyenSablon {
            tetik: cozum.tetik.clone(),
        })?;

    let baslangic = Instant::now();
    let genisletme = baglam.motor.genislet_sablon(&depo, &sablon, &degerler)?;
    let sure = baslangic.elapsed();

    let kazanc = crate::kazanc::olc(&genisletme.metin, &cozum.tetik_yazimi, sure, 1.0);
    if sablon.gizli {
        let maskeli = gizli::tumunu_maskele(&genisletme.metin);
        satir_yaz(&maskeli)?;
    } else {
        satir_yaz(&genisletme.metin)?;
    }
    if args.gerekce {
        satir_yaz(&format!(
            "# cozum: tetik={} gerekce={} adim={} derinlik={} sure={} us",
            cozum.tetik,
            gerekce_adi(cozum.gerekce),
            genisletme.adim,
            genisletme.derinlik,
            sure.as_micros()
        ))?;
        satir_yaz(&format!(
            "# kazanc: gerekli={} tus gercek={} tus kazanc={} ({})",
            kazanc.gerekli_tus,
            kazanc.gercek_tus,
            kazanc.kazanc,
            if kazanc.zararli() {
                "zararli"
            } else {
                "kazanc"
            }
        ))?;
    }

    if !baglam.oturum {
        pano_kaydet_ve_say(baglam, &sablon, &genisletme.metin, &kazanc)?;
    }
    Ok(0)
}

fn gerekce_adi(g: crate::cozumle::CozumGerekcesi) -> &'static str {
    match g {
        crate::cozumle::CozumGerekcesi::Tek => "tek-aday",
        crate::cozumle::CozumGerekcesi::EnUzun => "en-uzun",
        crate::cozumle::CozumGerekcesi::TamSayi => "tam-sayi",
        crate::cozumle::CozumGerekcesi::EnSikKullanim => "en-sik-kullanim",
    }
}

fn pano_kaydet_ve_say(baglam: &Baglam, sablon: &Sablon, metin: &str, kazanc: &Kazanc) -> Sonuc<()> {
    let mut pano = depo::panoyu_yukle(&baglam.yol)?;
    pano.ekle(metin, 0);
    kaydet_pano(baglam, &pano)?;

    let mut depo = depo::sablonlari_yukle(&baglam.yol)?;
    depo.kullanim_artir(&sablon.tetik)?;
    kaydet_sablonlar(baglam, &depo)?;

    let mut istatistik = depo::istatistigi_yukle(&baglam.yol)?;
    istatistik.ekle(&sablon.tetik, *kazanc, VARSAYILAN_TUS_CARPANI);
    kaydet_istatistik(baglam, &istatistik)?;
    Ok(())
}

fn pano_calistir(baglam: &Baglam, komut: &PanoKomutu) -> Sonuc<i32> {
    match komut {
        PanoKomutu::Ekle { metin } => {
            let mut pano = depo::panoyu_yukle(&baglam.yol)?;
            let kabul = pano.ekle(metin, 0);
            kaydet_pano(baglam, &pano)?;
            let mesaj = match kabul {
                PanoKabul::Eklendi(id) => format!("eklendi: kayit #{id}"),
                PanoKabul::Tekrar(id) => format!("zaten vardi, basa tasindi: kayit #{id}"),
                PanoKabul::Atlandi => format!(
                    "atlandi (boyut siniri {} karakter); toplam atlanan: {}",
                    pano.maks_boyut, pano.atlanan
                ),
            };
            satir_yaz(&mesaj)?;
            Ok(0)
        }
        PanoKomutu::Oku { id, coz } => {
            let pano = depo::panoyu_yukle(&baglam.yol)?;
            let kayit = pano
                .id_ile(*id)
                .ok_or_else(|| TypeFastHata::BilinmeyenSablon {
                    tetik: format!("pano kaydi #{id}"),
                })?;
            if *coz {
                satir_yaz(&kayit.metin)?;
            } else {
                satir_yaz(&kayit.gorunen_metin())?;
            }
            Ok(0)
        }
        PanoKomutu::Liste { adet, gizliler } => {
            let pano = depo::panoyu_yukle(&baglam.yol)?;
            if pano.bos() {
                satir_yaz("pano gecmisi bos")?;
                return Ok(0);
            }
            satir_yaz(&format!(
                "{} kayit (sinir {}), {} atlandi",
                pano.uzunluk(),
                pano.kapasite,
                pano.atlanan
            ))?;
            for kayit in pano.listele().iter().take(*adet) {
                if kayit.gizli && !gizliler {
                    satir_yaz(&format!(
                        "{:>4}  {:>5}  [gizli] {}",
                        kayit.id,
                        kayit.boyut,
                        gizli::tumunu_maskele(&kayit.metin)
                    ))?;
                } else {
                    satir_yaz(&format!(
                        "{:>4}  {:>5}  {}",
                        kayit.id,
                        kayit.boyut,
                        kayit.gorunen_metin()
                    ))?;
                }
            }
            Ok(0)
        }
        PanoKomutu::Ara { sorgu } => {
            let pano = depo::panoyu_yukle(&baglam.yol)?;
            let sonuc = pano.ara(sorgu, false);
            satir_yaz(&format!("{} sonuc", sonuc.len()))?;
            for kayit in sonuc {
                satir_yaz(&format!("{:>4}  {}", kayit.id, kayit.gorunen_metin()))?;
            }
            Ok(0)
        }
        PanoKomutu::Temizle => {
            let mut pano = depo::panoyu_yukle(&baglam.yol)?;
            let silinen = pano.uzunluk();
            pano.temizle();
            kaydet_pano(baglam, &pano)?;
            satir_yaz(&format!("silindi: {silinen} kayit"))?;
            Ok(0)
        }
    }
}

fn ara_calistir(baglam: &Baglam, args: &Ara) -> Sonuc<i32> {
    let depo = depo::sablonlari_yukle(&baglam.yol)?;
    let cozumleyici = Cozumleyici::yeni(&depo);
    let eslesmeler = cozumleyici.tum_eslesmeler(&args.sorgu);
    satir_yaz(&format!("sablon: {} eslesme", eslesmeler.len()))?;
    for cozum in &eslesmeler {
        satir_yaz(&format!(
            "  @{}  {} ({})",
            cozum.baslangic,
            cozum.tetik,
            gerekce_adi(cozum.gerekce)
        ))?;
    }
    if args.pano {
        let pano = depo::panoyu_yukle(&baglam.yol)?;
        let bulunan = pano.ara(&args.sorgu, false);
        satir_yaz(&format!("pano: {} sonuc", bulunan.len()))?;
        for kayit in bulunan {
            satir_yaz(&format!("  #{}  {}", kayit.id, kayit.gorunen_metin()))?;
        }
    }
    Ok(0)
}

fn export_calistir(baglam: &Baglam, args: &Aktar) -> Sonuc<i32> {
    let bicim = Bicim::ayikla(&args.bicim)?;
    let depo = depo::sablonlari_yukle(&baglam.yol)?;
    let icerik = aktarma::disa_aktar(&depo, bicim)?;
    match &args.dosya {
        Some(yol) => {
            aktarma::dosyaya_yaz(yol, &icerik)?;
            satir_yaz(&format!(
                "yazildi: {} ({} sablon, {} bayt)",
                yol.display(),
                depo.uzunluk(),
                icerik.len()
            ))?;
        }
        None => {
            let mut c = cikti();
            let _ = writeln!(c, "{icerik}");
        }
    }
    Ok(0)
}

fn import_calistir(baglam: &Baglam, args: &Al) -> Sonuc<i32> {
    let icerik = match &args.dosya {
        Some(yol) => std::fs::read_to_string(yol).map_err(|hata| TypeFastHata::OkumaHatasi {
            yol: yol.clone(),
            hata,
        })?,
        None => {
            let mut metin = String::new();
            std::io::stdin()
                .read_to_string(&mut metin)
                .map_err(|hata| TypeFastHata::OkumaHatasi {
                    yol: PathBuf::from("<stdin>"),
                    hata,
                })?;
            metin
        }
    };
    let bicim = if args.bicim == "otomatik" {
        aktarma::bicim_sezgile(&icerik)
    } else {
        Bicim::ayikla(&args.bicim)?
    };
    let gelen = aktarma::ice_aktar(&icerik, bicim)?;
    let mut depo = if args.degistir {
        crate::sablon::SablonDeposu::yeni()
    } else {
        depo::sablonlari_yukle(&baglam.yol)?
    };
    let mut eklenen = 0usize;
    let mut atlanan = 0usize;
    for sablon in gelen.tumune() {
        if depo.ekle(sablon.clone()).is_ok() {
            eklenen += 1;
        } else {
            atlanan += 1;
        }
    }
    kaydet_sablonlar(baglam, &depo)?;
    satir_yaz(&format!(
        "bicim: {} | eklendi: {eklenen} | atlandi: {atlanan}",
        bicim.ad()
    ))?;
    Ok(0)
}

fn stats_calistir(baglam: &Baglam, args: &Stats) -> Sonuc<i32> {
    let istatistik = depo::istatistigi_yukle(&baglam.yol)?;
    let pano = depo::panoyu_yukle(&baglam.yol)?;
    let depo_sablon = depo::sablonlari_yukle(&baglam.yol)?;
    satir_yaz("== typefast olcum raporu ==")?;
    satir_yaz(&format!("tetik sayisi      : {}", depo_sablon.uzunluk()))?;
    satir_yaz(&format!("pano kaydi        : {}", pano.uzunluk()))?;
    satir_yaz(&format!("pano siniri       : {}", pano.kapasite))?;
    satir_yaz(&format!("atlanan kayit     : {}", pano.atlanan))?;
    satir_yaz(&format!(
        "gizli isaretli kayit: {}",
        pano.gizli_kayitlar().len()
    ))?;
    satir_yaz(&format!("tus carpani       : {:.2}", args.tus_carpani))?;
    if istatistik.olculmedi() {
        satir_yaz("genisleme         : OLCULMEDI (hic genisletme yapilmadi)")?;
        satir_yaz("kazanilan tus     : OLCULMEDI")?;
        satir_yaz("kazanilan karakter: OLCULMEDI")?;
        satir_yaz("toplam sure       : OLCULMEDI")?;
        return Ok(0);
    }
    satir_yaz(&format!("genisleme         : {}", istatistik.genisletme))?;
    satir_yaz(&format!(
        "kazanilan tus     : {} (carpan {:.2} uygulanmis)",
        (istatistik.kazanilan_tus as f64 * args.tus_carpani).round() as i64,
        args.tus_carpani
    ))?;
    satir_yaz(&format!(
        "kazanilan karakter: {}",
        istatistik.kazanilan_karakter
    ))?;
    satir_yaz(&format!(
        "toplam sure       : {:.6} sn",
        istatistik.gecen_sn
    ))?;
    match istatistik.ortalama_sn() {
        Some(ortalama) => satir_yaz(&format!("ortalama sure     : {:.6} sn", ortalama))?,
        None => satir_yaz("ortalama sure     : OLCULMEDI")?,
    }
    let en_cok = istatistik.en_cok_kazanan(args.en_cok);
    if en_cok.is_empty() {
        satir_yaz("en cok kazandiran: yok")?;
    } else {
        satir_yaz("en cok kazandiran:")?;
        for (tetik, toplam) in en_cok {
            satir_yaz(&format!(
                "  {:<16} adet={:<4} kazanc={}",
                tetik, toplam.adet, toplam.kazanc
            ))?;
        }
    }
    Ok(0)
}

fn mask_calistir(args: &Maskele) -> Sonuc<i32> {
    let sonuc = gizli::maskele(&args.metin);
    if args.tur {
        let turler = sonuc.turler();
        if turler.is_empty() {
            satir_yaz("sir kalibi bulunamadi")?;
        } else {
            for tur in turler {
                satir_yaz(tur.ad())?;
            }
        }
        return Ok(0);
    }
    satir_yaz(&sonuc.metin)?;
    satir_yaz(&format!(
        "# bulgu: {} (kaynak metin DEGISMEDI; yalniz goruntulenen metin maskelendi)",
        sonuc.bulgular.len()
    ))?;
    Ok(0)
}

fn conflicts_calistir(baglam: &Baglam, args: &Cakisma) -> Sonuc<i32> {
    let depo = depo::sablonlari_yukle(&baglam.yol)?;
    let cozumleyici = Cozumleyici::yeni(&depo);
    let ornekler = ["a", "is", ";is", "is parcasi"];
    let mut bulunan: Vec<(String, Vec<String>, String)> = Vec::new();
    for ornek in ornekler {
        if let Err(TypeFastHata::CozumBelirsiz { adaylar, .. }) = cozumleyici.coz(ornek) {
            let ozet = adaylar.join(", ");
            bulunan.push((ornek.to_string(), adaylar, ozet));
        }
    }
    match args.bicim.as_str() {
        "json" => {
            satir_yaz(&format!("{}", bulunan.len()))?;
            for (ornek, _, ozet) in &bulunan {
                satir_yaz(&format!("  {ornek} -> {ozet}"))?;
            }
        }
        _ => {
            if bulunan.is_empty() {
                satir_yaz("belirsiz tetik yok")?;
            } else {
                for (ornek, _, ozet) in &bulunan {
                    satir_yaz(&format!("{ornek} -> {ozet}"))?;
                }
            }
        }
    }
    Ok(0)
}

fn yollar_calistir(baglam: &Baglam) -> Sonuc<i32> {
    satir_yaz(&format!("kok        : {}", baglam.yol.kok.display()))?;
    satir_yaz(&format!(
        "sablonlar  : {}",
        baglam.yol.sablon_dosyasi().display()
    ))?;
    satir_yaz(&format!(
        "pano       : {}",
        baglam.yol.pano_dosyasi().display()
    ))?;
    satir_yaz(&format!(
        "istatistik : {}",
        baglam.yol.istatistik_dosyasi().display()
    ))?;
    satir_yaz(&format!(
        "oturum     : {}",
        if baglam.oturum {
            "acik (diske yazilmaz)"
        } else {
            "kapali"
        }
    ))?;
    Ok(0)
}
