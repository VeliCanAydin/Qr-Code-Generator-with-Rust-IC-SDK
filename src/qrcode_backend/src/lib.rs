use candid::{CandidType, Deserialize};
use std::include_bytes;

mod core;

// QR kodu görüntüsünün boyutu (piksel cinsinden)
const IMAGE_SIZE_IN_PIXELS: usize = 1024;
// Şeffaf logo dosyasının veri içeriği
const LOGO_TRANSPARENT: &[u8] = include_bytes!("./assets/logo_transparent.png");
// Beyaz logo dosyasının veri içeriği
const LOGO_WHITE: &[u8] = include_bytes!("./assets/logo_white.png");

// Options yapısı, CandidType ve Deserialize öznitelikleri ile işaretlenmiştir.
#[derive(CandidType, Deserialize)]
struct Options {
    add_logo: bool,                 // Logonun eklenip eklenmeyeceği
    add_gradient: bool,             // Gradyanın eklenip eklenmeyeceği
    add_transparency: Option<bool>, // Şeffaflığın eklenip eklenmeyeceği (opsiyonel)
}

// Hata mesajını içeren bir yapı
#[derive(CandidType, Deserialize)]
struct QrError {
    message: String,
}

// QrResult, CandidType ve Deserialize öznitelikleri ile işaretlenmiş bir enum
#[derive(CandidType, Deserialize)]
enum QrResult {
    Image(Vec<u8>), // QR kodu görüntüsü veri içeriği
    Err(QrError),   // Hata durumunda mesaj içeriği
}

// QR kodu üretimini gerçekleştiren işlev
fn qrcode_impl(input: String, options: Options) -> QrResult {
    // Şeffaflık opsiyonuna göre uygun logo belirlenir
    let logo = if options.add_transparency == Some(true) {
        LOGO_TRANSPARENT
    } else {
        LOGO_WHITE
    };

    // QR kodu üretim işlemi çağrılır
    let result = match core::generate(input, options, logo, IMAGE_SIZE_IN_PIXELS) {
        Ok(blob) => QrResult::Image(blob), // Başarılı sonuçta QR kodu görüntüsü
        Err(err) => QrResult::Err(QrError {
            // Hata durumunda hata mesajı
            message: err.to_string(),
        }),
    };

    // İşlem adım sayısını yazdır (IC performans izleyici kullanılarak)
    ic_cdk::println!(
        "Executed instructions: {}",
        ic_cdk::api::performance_counter(0)
    );

    result // Sonuç döndürülür
}

// IC update fonksiyonu: Mutating fonksiyon, state'i değiştirir
#[ic_cdk::update]
fn qrcode(input: String, options: Options) -> QrResult {
    qrcode_impl(input, options) // qrcode_impl işlevi çağrılır
}

// IC query fonksiyonu: State'i değiştirmez, sorgu yapar
#[ic_cdk::query]
fn qrcode_query(input: String, options: Options) -> QrResult {
    qrcode_impl(input, options) // qrcode_impl işlevi çağrılır
}
