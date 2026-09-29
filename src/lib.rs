//! TypeFast (YazHızlı) — genel metin genişletici ve pano geçmişi.
//!
//! # Ne yapar
//!
//! * **Şablon deposu** — tetik, gövde, kategori, açıklama ve kullanım sayacı.
//! * **Genişleme motoru** — `{{tarih}}`, `{{saat}}`, `{{isim}}` gibi yerleşikler,
//!   kullanıcı değerleri, iç içe şablon çağrısı, derinlik ve adım sınırları.
//! * **Kısayol çözümleme** — nokta, boşluk ve noktalama ile ayrılmış kelime
//!   dizileri; en uzun eşleşme, tam sayı ve en sık kullanım ölçütleri.
//! * **Pano geçmişi** — sınırlı halka tampon, otomatik eski temizleme, arama.
//! * **Gizli mod** — parola/token/anahtar kalıplarını tespit edip maskeleme.
//! * **Ölçülebilir kazanç** — gerçek tuş sayacı ve `Instant` süresi.
//! * **Dışa/içe aktarma** — JSON paket ve sekmeyle ayrılmış düz metin liste.
//!
//! # Ne yapmaz
//!
//! **Şifreleme yapmaz.** Pano geçmişi ve şablonlar düz metin JSON'dur.
//! Bu, ürünün en görünür eksikliğidir ve `README.md`'nin **en üstünde**
//! uyarı olarak yazılıdır (MANIFEST kart 25 — riskler).
//!
//! **Pano dinlemez, global kısayol kaydetmez.** Bu yüzden platform FFI'si
//! (`unsafe`) hiç kullanılmaz; etkileşim komut satırı ve dosya üzerinden olur.
//!
//! **Sözdizimi denetleyicisi yoktur.** Bu proje *genel metin* içindir
//! (MANIFEST karar D-011); programlama bilgisi gerektiren özellikler kardeş
//! proje SnipHub'ın kapsamındadır. Bu ayrım `README.md` → `## Özellikler`
//! bölümünde gerekçesiyle anlatılır.
//!
//! **Davranış öğrenmesi yoktur.** Rapor bunu `v1` özelliği olarak listeler;
//! karar, ölçülebilir kazancın ve gizliliğin ancak kural tabanlı kalınacağı
//! şeklinde uygulanmıştır. Gerekçe `README.md` → `## Bilinen Sınırlamalar`.
//!
//! # Modüller
//!
//! | Modül | Sorumluluk |
//! |---|---|
//! | [`sablon`] | Şablon varlığı, depo, çakışma denetimi, kullanım sayacı |
//! | [`cozumle`] | Tetik sözdizimi, metin taraması, belirsizlik çözümü |
//! | [`genislet`] | `{{ ... }}` genişleme motoru, iç içe çağrı, döngü koruması |
//! | [`gizli`] | Sır kalıbı tanıma ve maskeleme |
//! | [`pano`] | Sınırlı pano geçmişi halka tamponu |
//! | [`kazanc`] | Tuş sayacı tabanlı kazanç ölçümü ve istatistik |
//! | [`depo`] | JSON yükleme/kaydetme ve atomik yazma |
//! | [`aktarma`] | JSON paket ve düz metin liste dışa/içe aktarma |
//! | [`cli`] | `clap` komut satırı arayüzü |
//! | [`hata`] | Ortak hata tipi |

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

pub mod aktarma;
pub mod cli;
pub mod cozumle;
pub mod depo;
pub mod genislet;
pub mod gizli;
pub mod hata;
pub mod kazanc;
pub mod pano;
pub mod sablon;

/// Program sürümü (`Cargo.toml` ile eşleşir).
pub const SURUM: &str = env!("CARGO_PKG_VERSION");

/// Kısa ad; `README.md` ve yardım çıktısında kullanılır.
pub const AD: &str = "TypeFast";

/// Panoya eklenen kayıtların varsayılan üst sınırı.
pub const PANO_KAPASITE: usize = pano::VARSAYILAN_KAPASITE;
