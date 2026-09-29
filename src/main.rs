//! TypeFast ikili giriş noktası.
//!
//! Tüm mantık `typefast` kütüphanesindedir; burada yalnız `clap` ayrıştırma,
//! hata yazdırma ve çıkış kodu belirlenir. Bu ayrım sayesinde birim testleri
//! ikiliyi derlemeden çalışır (`WORKER_CONTRACT.md` § 1).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use clap::Parser;

use typefast::cli::{self, KomutSatiri};

fn main() -> ExitCode {
    let komut = KomutSatiri::parse();
    match cli::calistir(&komut) {
        Ok(kod) => ExitCode::from(u8::try_from(kod).unwrap_or(1)),
        Err(hata) => {
            eprintln!("typefast: {hata}");
            ExitCode::from(1)
        }
    }
}
