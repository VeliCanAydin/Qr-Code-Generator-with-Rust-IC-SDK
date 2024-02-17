use image::{imageops, ImageBuffer, Rgba};
use qrcode_generator::QrCodeEcc;
use std::io::Cursor;

use crate::Options;

/// Belirtilen giriş metni için PNG biçiminde bir QR kodu görüntüsü oluşturur.
/// İstenen görüntü boyutu pikseller cinsinden belirtilmelidir.
pub(super) fn generate(
    input: String,
    options: Options,
    logo: &[u8],
    image_size: usize,
) -> Result<Vec<u8>, anyhow::Error> {
    // 25% hata toleransı olan bir QR kodu görüntüsü oluşturun.
    let mut qr = image::DynamicImage::ImageLuma8(qrcode_generator::to_image_buffer(
        input,
        QrCodeEcc::Quartile,
        image_size,
    )?)
    .into_rgba8();

    if options.add_transparency == Some(true) {
        make_transparent(&mut qr);
    }

    if options.add_logo {
        add_logo(&mut qr, logo);
    }

    if options.add_gradient {
        add_gradient(&mut qr);
    }

    let mut result = vec![];
    qr.write_to(&mut Cursor::new(&mut result), image::ImageOutputFormat::Png)?;
    Ok(result)
}

/// Görüntüdeki beyaz pikselleri şeffaf piksellerle değiştirir.
fn make_transparent(qr: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    for (_x, _y, pixel) in qr.enumerate_pixels_mut() {
        if pixel.0 == [255, 255, 255, 255] {
            *pixel = image::Rgba([255, 255, 255, 0]);
        }
    }
}

/// Verilen logoyu QR kodu görüntüsünün merkezine ekler.
/// Logo, görüntünün %10'unu kaplamayacak şekilde boyutlandırılır,
/// bu QR hata eşiğinin altındadır.
fn add_logo(qr: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, logo: &[u8]) {
    let image_size = qr.width().min(qr.height()) as usize;
    let element_size = get_qr_element_size(qr);

    // En küçük kare boyutunu bulmak için logo boyutunu ayarlar.
    let mut logo_size = element_size;

    // İki eleman ekleyerek logonun görüntünün ortasında kalmasını sağlar.
    while logo_size + 2 * element_size <= 5 * image_size / 16 {
        logo_size += 2 * element_size;
    }

    let mut logo = image::io::Reader::new(Cursor::new(logo))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    logo = logo.resize(
        logo_size as u32,
        logo_size as u32,
        imageops::FilterType::Lanczos3,
    );

    imageops::replace(
        qr,
        &logo,
        ((image_size - logo_size) / 2) as i64,
        ((image_size - logo_size) / 2) as i64,
    );
}

/// QR kodu görüntüsünün siyah karelerine bir renk gradyanı ekler.
/// Gradyan, görüntünün merkezinden kenarlarına kadar gider.
fn add_gradient(qr: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let image_size = qr.width().min(qr.height()) as usize;

    // İki renge dayalı bir doğrusal gradyan işlevi hazırlar.
    // Her nokta `x` için `[0.0 .. 1.0]` aralığında, işlev
    // iki başlangıç rengi arasında bir renk döndürür: `gradient.at(x)`.
    let gradient = colorgrad::CustomGradient::new()
        .colors(&[
            colorgrad::Color::from_rgba8(100, 0, 100, 255),
            colorgrad::Color::from_rgba8(30, 5, 60, 255),
        ])
        .build()
        .unwrap();

    // Gradyan, görüntünün merkezinden kenarlarına kadar gider.
    let center = (image_size / 2) as u32;
    for (x, y, pixel) in qr.enumerate_pixels_mut() {
        if pixel.0 == [0, 0, 0, 255] {
            // Pikselin görüntünün merkezinden ne kadar uzak olduğunun bir tahmini olarak
            // basit Manhattan mesafesi kullanılır.
            let distance = x.abs_diff(center) + y.abs_diff(center);
            let rgba = gradient.at(distance as f64 / image_size as f64).to_rgba8();
            *pixel = image::Rgba(rgba);
        }
    }
}

/// Bir QR kodu görüntüsü verildiğinde, bu işlev görüntünün en küçük siyah karesinin boyutunu döndürür.
fn get_qr_element_size(qr: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> usize {
    const BLACK_PIXEL: [u8; 4] = [0, 0, 0, 255];

    let size = qr.width().min(qr.height());

    // Köşegen bir çizgi çizerek görüntüyü baştan çıkararak ilk siyah pikseli bulunur.
    let mut start = size;
    for i in 0..size {
        if qr.get_pixel(i, i).0 == BLACK_PIXEL {
            start = i;
            break;
        }
    }

    // Renk değişimine kadar köşegen geçişi devam ettirir.
    let mut element_size = 1;
    for i in 0..size - start {
        if qr.get_pixel(start + i, start + i).0 != BLACK_PIXEL {
            element_size = i;
            break;
        }
    }

    element_size as usize
}
