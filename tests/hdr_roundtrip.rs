use std::ffi::CStr;
use std::slice;

use jc_libavif_sys as ffi;

const WIDTH: usize = 4;
const HEIGHT: usize = 4;
const OPAQUE_PROPERTY_BOX: [u8; 4] = *b"msgr";
const OPAQUE_PROPERTY_PAYLOAD: &[u8] = b"jc-libavif-sys";
const NETFLIX_HDR10_YUV420_QP10: &[u8] =
    include_bytes!("fixtures/hdr_cosmos01000_cicp9-16-9_yuv420_limited_qp10.avif");
const NETFLIX_HDR10_YUV444_QP10: &[u8] =
    include_bytes!("fixtures/hdr_cosmos01000_cicp9-16-9_yuv444_full_qp10.avif");

#[test]
fn roundtrips_hdr10_pq_image() {
    roundtrip_hdr_image(
        10,
        ffi::AVIF_TRANSFER_CHARACTERISTICS_SMPTE2084 as ffi::avifTransferCharacteristics,
        1_000,
        400,
    );
}

#[test]
fn roundtrips_hdr12_hlg_image() {
    roundtrip_hdr_image(
        12,
        ffi::AVIF_TRANSFER_CHARACTERISTICS_HLG as ffi::avifTransferCharacteristics,
        4_000,
        1_000,
    );
}

#[test]
fn gain_map_api_is_available() {
    let gain_map = unsafe { ffi::avifGainMapCreate() };
    assert!(!gain_map.is_null(), "avifGainMapCreate returned NULL");
    unsafe {
        assert_eq!((*gain_map).altDepth, 0);
        ffi::avifGainMapDestroy(gain_map);
    }
}

#[test]
fn decodes_netflix_hdr10_yuv420_fixture() {
    decode_fixture_and_assert(
        NETFLIX_HDR10_YUV420_QP10,
        ffi::avifPixelFormat_AVIF_PIXEL_FORMAT_YUV420,
        ffi::avifRange_AVIF_RANGE_LIMITED,
        9,
    );
}

#[test]
fn decodes_netflix_hdr10_yuv444_fixture() {
    decode_fixture_and_assert(
        NETFLIX_HDR10_YUV444_QP10,
        ffi::avifPixelFormat_AVIF_PIXEL_FORMAT_YUV444,
        ffi::avifRange_AVIF_RANGE_FULL,
        9,
    );
}

fn roundtrip_hdr_image(
    depth: u32,
    transfer: ffi::avifTransferCharacteristics,
    max_cll: u16,
    max_pall: u16,
) {
    unsafe {
        let source = Image::new(ffi::avifImageCreate(
            WIDTH as u32,
            HEIGHT as u32,
            depth,
            ffi::avifPixelFormat_AVIF_PIXEL_FORMAT_YUV444,
        ));
        let source_image = source.as_mut();

        expect_ok(
            ffi::avifImageAllocatePlanes(source_image, ffi::avifPlanesFlag_AVIF_PLANES_YUV),
            "avifImageAllocatePlanes(source)",
        );
        configure_hdr_metadata(source_image, transfer, max_cll, max_pall);
        fill_test_pattern(source_image);

        expect_ok(
            ffi::avifImageAddOpaqueProperty(
                source_image,
                OPAQUE_PROPERTY_BOX.as_ptr(),
                OPAQUE_PROPERTY_PAYLOAD.as_ptr(),
                OPAQUE_PROPERTY_PAYLOAD.len(),
            ),
            "avifImageAddOpaqueProperty",
        );

        let mut encoder = Encoder::new(ffi::avifEncoderCreate());
        let encoder_ref = encoder.as_mut();
        encoder_ref.codecChoice = ffi::avifCodecChoice_AVIF_CODEC_CHOICE_AOM;
        encoder_ref.maxThreads = 1;
        encoder_ref.speed = ffi::AVIF_SPEED_SLOWEST as i32;
        encoder_ref.quality = ffi::AVIF_QUALITY_LOSSLESS as i32;
        encoder_ref.qualityAlpha = ffi::AVIF_QUALITY_LOSSLESS as i32;
        encoder_ref.qualityGainMap = ffi::AVIF_QUALITY_LOSSLESS as i32;

        let mut encoded = OwnedRwData::default();
        expect_ok(
            ffi::avifEncoderWrite(encoder.as_ptr(), source_image, encoded.as_mut_ptr()),
            "avifEncoderWrite",
        );
        assert!(
            encoded.as_slice().len() > 32,
            "encoded AVIF payload is unexpectedly small"
        );

        let decoded = Image::new(ffi::avifImageCreateEmpty());
        let mut decoder = Decoder::new(ffi::avifDecoderCreate());
        let decoder_ref = decoder.as_mut();
        decoder_ref.codecChoice = ffi::avifCodecChoice_AVIF_CODEC_CHOICE_AOM;
        decoder_ref.maxThreads = 1;
        decoder_ref.imageContentToDecode =
            ffi::avifImageContentTypeFlag_AVIF_IMAGE_CONTENT_DECODE_DEFAULT;

        expect_ok(
            ffi::avifDecoderReadMemory(
                decoder.as_ptr(),
                decoded.as_ptr(),
                encoded.as_slice().as_ptr(),
                encoded.as_slice().len(),
            ),
            "avifDecoderReadMemory",
        );

        let decoded_image = decoded.as_ref();
        assert_eq!(decoded_image.width, WIDTH as u32);
        assert_eq!(decoded_image.height, HEIGHT as u32);
        assert_eq!(decoded_image.depth, depth);
        assert_eq!(
            decoded_image.colorPrimaries,
            ffi::AVIF_COLOR_PRIMARIES_BT2020 as ffi::avifColorPrimaries
        );
        assert_eq!(decoded_image.transferCharacteristics, transfer);
        assert_eq!(
            decoded_image.matrixCoefficients,
            ffi::AVIF_MATRIX_COEFFICIENTS_BT2020_NCL as ffi::avifMatrixCoefficients
        );
        assert_eq!(decoded_image.yuvRange, ffi::avifRange_AVIF_RANGE_FULL);
        assert_eq!(decoded_image.clli.maxCLL, max_cll);
        assert_eq!(decoded_image.clli.maxPALL, max_pall);
        assert_eq!(decoded_image.numProperties, 1);

        let property = &*decoded_image.properties;
        assert_eq!(property.boxtype, OPAQUE_PROPERTY_BOX);
        assert_eq!(
            slice::from_raw_parts(property.boxPayload.data, property.boxPayload.size),
            OPAQUE_PROPERTY_PAYLOAD
        );

        assert_yuv_match(source.as_ref(), decoded_image);

        let mut rgb = OwnedRgbImage::new(decoded_image);
        rgb.as_mut().depth = 16;
        rgb.as_mut().format = ffi::avifRGBFormat_AVIF_RGB_FORMAT_RGBA;
        rgb.as_mut().chromaUpsampling =
            ffi::avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_BEST_QUALITY;
        rgb.as_mut().maxThreads = 1;
        expect_ok(
            ffi::avifRGBImageAllocatePixels(rgb.as_mut()),
            "avifRGBImageAllocatePixels",
        );
        expect_ok(
            ffi::avifImageYUVToRGB(decoded_image, rgb.as_mut()),
            "avifImageYUVToRGB",
        );

        let rgb_words =
            slice::from_raw_parts(rgb.as_ref().pixels.cast::<u16>(), WIDTH * HEIGHT * 4);
        assert!(
            rgb_words.iter().any(|sample| *sample > 0),
            "decoded RGB image should contain non-zero samples"
        );
    }
}

fn decode_fixture_and_assert(
    bytes: &[u8],
    expected_yuv_format: ffi::avifPixelFormat,
    expected_range: ffi::avifRange,
    expected_matrix: ffi::avifMatrixCoefficients,
) {
    unsafe {
        let decoded = Image::new(ffi::avifImageCreateEmpty());
        let mut decoder = Decoder::new(ffi::avifDecoderCreate());
        let decoder_ref = decoder.as_mut();
        decoder_ref.codecChoice = ffi::avifCodecChoice_AVIF_CODEC_CHOICE_AOM;
        decoder_ref.maxThreads = 1;
        decoder_ref.imageContentToDecode =
            ffi::avifImageContentTypeFlag_AVIF_IMAGE_CONTENT_DECODE_DEFAULT;

        expect_ok(
            ffi::avifDecoderReadMemory(
                decoder.as_ptr(),
                decoded.as_ptr(),
                bytes.as_ptr(),
                bytes.len(),
            ),
            "avifDecoderReadMemory(fixture)",
        );

        let image = decoded.as_ref();
        assert_eq!(image.width, 2048);
        assert_eq!(image.height, 858);
        assert_eq!(image.depth, 10);
        assert_eq!(image.yuvFormat, expected_yuv_format);
        assert_eq!(image.yuvRange, expected_range);
        assert_eq!(
            image.colorPrimaries,
            ffi::AVIF_COLOR_PRIMARIES_BT2020 as ffi::avifColorPrimaries
        );
        assert_eq!(
            image.transferCharacteristics,
            ffi::AVIF_TRANSFER_CHARACTERISTICS_SMPTE2084 as ffi::avifTransferCharacteristics
        );
        assert_eq!(image.matrixCoefficients, expected_matrix);
        assert!(
            image.gainMap.is_null(),
            "fixture unexpectedly contains a gain map"
        );

        let mut rgb = OwnedRgbImage::new(image);
        rgb.as_mut().depth = 16;
        rgb.as_mut().format = ffi::avifRGBFormat_AVIF_RGB_FORMAT_RGBA;
        rgb.as_mut().chromaUpsampling =
            ffi::avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_BEST_QUALITY;
        rgb.as_mut().maxThreads = 1;
        expect_ok(
            ffi::avifRGBImageAllocatePixels(rgb.as_mut()),
            "avifRGBImageAllocatePixels(fixture)",
        );
        expect_ok(
            ffi::avifImageYUVToRGB(image, rgb.as_mut()),
            "avifImageYUVToRGB(fixture)",
        );
        let rgb_words = slice::from_raw_parts(rgb.as_ref().pixels.cast::<u16>(), 2048 * 858 * 4);
        assert!(
            rgb_words.iter().any(|sample| *sample > 0),
            "fixture decode should produce non-zero RGB samples"
        );
    }
}

fn configure_hdr_metadata(
    image: *mut ffi::avifImage,
    transfer: ffi::avifTransferCharacteristics,
    max_cll: u16,
    max_pall: u16,
) {
    unsafe {
        (*image).yuvRange = ffi::avifRange_AVIF_RANGE_FULL;
        (*image).colorPrimaries = ffi::AVIF_COLOR_PRIMARIES_BT2020 as ffi::avifColorPrimaries;
        (*image).transferCharacteristics = transfer;
        (*image).matrixCoefficients =
            ffi::AVIF_MATRIX_COEFFICIENTS_BT2020_NCL as ffi::avifMatrixCoefficients;
        (*image).clli = ffi::avifContentLightLevelInformationBox {
            maxCLL: max_cll,
            maxPALL: max_pall,
        };
    }
}

fn fill_test_pattern(image: *mut ffi::avifImage) {
    let max = unsafe { ((1u32 << (*image).depth) - 1) as u16 };
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let luma =
                (((x + y * WIDTH) as u32 * max as u32) / ((WIDTH * HEIGHT - 1) as u32)) as u16;
            let u = (((x * 3 + y) as u32 * max as u32) / ((WIDTH * 3 + HEIGHT - 2) as u32)) as u16;
            let v = (((y * 5 + x) as u32 * max as u32) / ((HEIGHT * 5 + WIDTH - 2) as u32)) as u16;
            unsafe {
                write_sample(image, 0, x, y, luma);
                write_sample(image, 1, x, y, u);
                write_sample(image, 2, x, y, v);
            }
        }
    }
}

fn assert_yuv_match(expected: &ffi::avifImage, actual: &ffi::avifImage) {
    for plane in 0..3 {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let (expected_sample, actual_sample) = unsafe {
                    (
                        read_sample(expected, plane, x, y),
                        read_sample(actual, plane, x, y),
                    )
                };
                assert_eq!(
                    actual_sample, expected_sample,
                    "plane {plane} differs at ({x}, {y})"
                );
            }
        }
    }
}

unsafe fn write_sample(image: *mut ffi::avifImage, plane: usize, x: usize, y: usize, value: u16) {
    let row = unsafe { (*image).yuvPlanes[plane].add(y * (*image).yuvRowBytes[plane] as usize) };
    unsafe {
        row.cast::<u16>().add(x).write(value);
    }
}

unsafe fn read_sample(image: &ffi::avifImage, plane: usize, x: usize, y: usize) -> u16 {
    let row = unsafe { image.yuvPlanes[plane].add(y * image.yuvRowBytes[plane] as usize) };
    unsafe { row.cast::<u16>().add(x).read() }
}

fn expect_ok(result: ffi::avifResult, context: &str) {
    if result != ffi::avifResult_AVIF_RESULT_OK {
        let message = unsafe {
            CStr::from_ptr(ffi::avifResultToString(result))
                .to_string_lossy()
                .into_owned()
        };
        panic!("{context} failed: {message} ({result})");
    }
}

struct Image(*mut ffi::avifImage);

impl Image {
    fn new(ptr: *mut ffi::avifImage) -> Self {
        assert!(!ptr.is_null(), "libavif returned a null avifImage");
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut ffi::avifImage {
        self.0
    }

    fn as_mut(&self) -> *mut ffi::avifImage {
        self.0
    }

    fn as_ref(&self) -> &ffi::avifImage {
        unsafe { &*self.0 }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            ffi::avifImageDestroy(self.0);
        }
    }
}

struct Encoder(*mut ffi::avifEncoder);

impl Encoder {
    fn new(ptr: *mut ffi::avifEncoder) -> Self {
        assert!(!ptr.is_null(), "libavif returned a null avifEncoder");
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut ffi::avifEncoder {
        self.0
    }

    fn as_mut(&mut self) -> &mut ffi::avifEncoder {
        unsafe { &mut *self.0 }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            ffi::avifEncoderDestroy(self.0);
        }
    }
}

struct Decoder(*mut ffi::avifDecoder);

impl Decoder {
    fn new(ptr: *mut ffi::avifDecoder) -> Self {
        assert!(!ptr.is_null(), "libavif returned a null avifDecoder");
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut ffi::avifDecoder {
        self.0
    }

    fn as_mut(&mut self) -> &mut ffi::avifDecoder {
        unsafe { &mut *self.0 }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            ffi::avifDecoderDestroy(self.0);
        }
    }
}

#[derive(Default)]
struct OwnedRwData(ffi::avifRWData);

impl OwnedRwData {
    fn as_mut_ptr(&mut self) -> *mut ffi::avifRWData {
        &mut self.0
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.data, self.0.size) }
    }
}

impl Drop for OwnedRwData {
    fn drop(&mut self) {
        unsafe {
            ffi::avifRWDataFree(&mut self.0);
        }
    }
}

struct OwnedRgbImage(ffi::avifRGBImage);

impl OwnedRgbImage {
    fn new(image: &ffi::avifImage) -> Self {
        let mut rgb = ffi::avifRGBImage::default();
        unsafe {
            ffi::avifRGBImageSetDefaults(&mut rgb, image);
        }
        Self(rgb)
    }

    fn as_ref(&self) -> &ffi::avifRGBImage {
        &self.0
    }

    fn as_mut(&mut self) -> &mut ffi::avifRGBImage {
        &mut self.0
    }
}

impl Drop for OwnedRgbImage {
    fn drop(&mut self) {
        unsafe {
            ffi::avifRGBImageFreePixels(&mut self.0);
        }
    }
}
