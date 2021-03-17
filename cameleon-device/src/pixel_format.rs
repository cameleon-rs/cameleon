use std::convert::TryFrom;

#[allow(clippy::enum_glob_use)]
use PixelFormat::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Monochrome 8-bit.
    Mono8,

    /// Monochrome 8-bit signed.
    Mono8s,

    /// Monochrome 10-bit unpacked.
    Mono10,

    /// Monochrome 10-bit packed.
    Mono10Packed,

    /// Monochrome 12-bit unpacked.
    Mono12,

    /// Monochrome 12-bit packed.
    Mono12Packed,

    /// Monochrome 16-bit.
    Mono16,

    /// Bayer Green-Red 8-bit.
    BayerGR8,

    /// Bayer Red-Green 8-bit.
    BayerRG8,

    /// Bayer Green-Blue 8-bit.
    BayerGB8,

    /// Bayer Blue-Green 8-bit.
    BayerBG8,

    /// Bayer Green-Red 10-bit unpacked.
    BayerGR10,

    /// Bayer Red-Green 10-bit unpacked.
    BayerRG10,

    /// Bayer Green-Blue 10-bit unpacked.
    BayerGB10,

    /// Bayer Blue-Green 10-bit unpacked.
    BayerBG10,

    /// Bayer Green-Red 12-bit unpacked.
    BayerGR12,

    /// Bayer Red-Green 12-bit unpacked.
    BayerRG12,

    /// Bayer Green-Blue 12-bit unpacked.
    BayerGB12,

    /// Bayer Blue-Green 12-bit unpacked.
    BayerBG12,

    /// Red-Green-Blue 8-bit.
    RGB8,

    /// Blue-Green-Red 8-bit.
    BGR8,

    /// Red-Green-Blue-alpha 8-bit.
    RGBa8,

    /// Blue-Green-Red-alpha 8-bit.
    BGRa8,

    /// Red-Green-Blue 10-bit unpacked.
    RGB10,

    /// Blue-Green-Red 10-bit unpacked.
    BGR10,

    /// Red-Green-Blue 12-bit unpacked.
    RGB12,

    /// Blue-Green-Red 12-bit unpacked.
    BGR12,

    /// YUV 4:4:4 8-bit.
    YUV8_UYV,

    /// Red-Green-Blue 8-bit planar.
    RGB8_Planar,

    /// Red-Green-Blue 10-bit unpacked planar.
    RGB10_Planar,

    /// Red-Green-Blue 12-bit unpacked planar.
    RGB12_Planar,

    /// Red-Green-Blue 16-bit planar.
    RGB16_Planar,

    /// Monochrome 14-bit unpacked.
    Mono14,

    /// Bayer Green-Red 10-bit packed.
    BayerGR10Packed,

    /// Bayer Red-Green 10-bit packed.
    BayerRG10Packed,

    /// Bayer Green-Blue 10-bit packed.
    BayerGB10Packed,

    /// Bayer Blue-Green 10-bit packed.
    BayerBG10Packed,

    /// Bayer Green-Red 12-bit packed.
    BayerGR12Packed,

    /// Bayer Red-Green 12-bit packed.
    BayerRG12Packed,

    /// Bayer Green-Blue 12-bit packed.
    BayerGB12Packed,

    /// Bayer Blue-Green 12-bit packed.
    BayerBG12Packed,

    /// Bayer Green-Red 16-bit.
    BayerGR16,

    /// Bayer Red-Green 16-bit.
    BayerRG16,

    /// Bayer Green-Blue 16-bit.
    BayerGB16,

    /// Bayer Blue-Green 16-bit.
    BayerBG16,

    /// YUV 4:2:2 8-bit.
    YUV422_8,

    /// Red-Green-Blue 16-bit.
    RGB16,

    /// Red-Green-Blue 12-bit packed - variant 1.
    RGB12V1Packed,

    /// Red-Green-Blue 5/6/5-bit packed.
    RGB565p,

    /// Blue-Green-Red 5/6/5-bit packed.
    BGR565p,

    /// Monochrome 1-bit packed.
    Mono1p,

    /// Monochrome 2-bit packed.
    Mono2p,

    /// Monochrome 4-bit packed.
    Mono4p,

    /// YCbCr 4:4:4 8-bit.
    YCbCr8_CbYCr,

    /// YCbCr 4:2:2 8-bit.
    YCbCr422_8,

    /// YCbCr 4:1:1 8-bit.
    YCbCr411_8_CbYYCrYY,

    /// YCbCr 4:4:4 8-bit BT.601.
    YCbCr601_8_CbYCr,

    /// YCbCr 4:2:2 8-bit BT.601.
    YCbCr601_422_8,

    /// YCbCr 4:1:1 8-bit BT.601.
    YCbCr601_411_8_CbYYCrYY,

    /// YCbCr 4:4:4 8-bit BT.709.
    YCbCr709_8_CbYCr,

    /// YCbCr 4:2:2 8-bit BT.709.
    YCbCr709_422_8,

    /// YCbCr 4:1:1 8-bit BT.709.
    YCbCr709_411_8_CbYYCrYY,

    /// YCbCr 4:2:2 8-bit.
    YCbCr422_8_CbYCrY,

    /// YCbCr 4:2:2 8-bit BT.601.
    YCbCr601_422_8_CbYCrY,

    /// YCbCr 4:2:2 8-bit BT.709.
    YCbCr709_422_8_CbYCrY,

    /// Monochrome 10-bit packed.
    Mono10p,

    /// Monochrome 12-bit packed.
    Mono12p,

    /// Blue-Green-Red 10-bit packed.
    BGR10p,

    /// Blue-Green-Red 12-bit packed.
    BGR12p,

    /// Blue-Green-Red 14-bit unpacked.
    BGR14,

    /// Blue-Green-Red 16-bit.
    BGR16,

    /// Blue-Green-Red-alpha 10-bit unpacked.
    BGRa10,

    /// Blue-Green-Red-alpha 10-bit packed.
    BGRa10p,

    /// Blue-Green-Red-alpha 12-bit unpacked.
    BGRa12,

    /// Blue-Green-Red-alpha 12-bit packed.
    BGRa12p,

    /// Blue-Green-Red-alpha 14-bit unpacked.
    BGRa14,

    /// Blue-Green-Red-alpha 16-bit.
    BGRa16,

    /// Bayer Blue-Green 10-bit packed.
    BayerBG10p,

    /// Bayer Blue-Green 12-bit packed.
    BayerBG12p,

    /// Bayer Green-Blue 10-bit packed.
    BayerGB10p,

    /// Bayer Green-Blue 12-bit packed.
    BayerGB12p,

    /// Bayer Green-Red 10-bit packed.
    BayerGR10p,

    /// Bayer Green-Red 12-bit packed.
    BayerGR12p,

    /// Bayer Red-Green 10-bit packed.
    BayerRG10p,

    /// Bayer Red-Green 12-bit packed.
    BayerRG12p,

    /// YCbCr 4:1:1 8-bit.
    YCbCr411_8,

    /// YCbCr 4:4:4 8-bit.
    YCbCr8,

    /// Red-Green-Blue 10-bit packed.
    RGB10p,

    /// Red-Green-Blue 12-bit packed.
    RGB12p,

    /// Red-Green-Blue 14-bit unpacked.
    RGB14,

    /// Red-Green-Blue-alpha 10-bit unpacked.
    RGBa10,

    /// Red-Green-Blue-alpha 10-bit packed.
    RGBa10p,

    /// Red-Green-Blue-alpha 12-bit unpacked.
    RGBa12,

    /// Red-Green-Blue-alpha 12-bit packed.
    RGBa12p,

    /// Red-Green-Blue-alpha 14-bit unpacked.
    RGBa14,

    /// Red-Green-Blue-alpha 16-bit.
    RGBa16,

    /// YCbCr 4:2:2 10-bit unpacked.
    YCbCr422_10,

    /// YCbCr 4:2:2 12-bit unpacked.
    YCbCr422_12,

    /// Sparse Color Filter #1 White-Blue-White-Green 8-bit.
    SCF1WBWG8,

    /// Sparse Color Filter #1 White-Blue-White-Green 10-bit unpacked.
    SCF1WBWG10,

    /// Sparse Color Filter #1 White-Blue-White-Green 10-bit packed.
    SCF1WBWG10p,

    /// Sparse Color Filter #1 White-Blue-White-Green 12-bit unpacked.
    SCF1WBWG12,

    /// Sparse Color Filter #1 White-Blue-White-Green 12-bit packed.
    SCF1WBWG12p,

    /// Sparse Color Filter #1 White-Blue-White-Green 14-bit unpacked.
    SCF1WBWG14,

    /// Sparse Color Filter #1 White-Blue-White-Green 16-bit unpacked.
    SCF1WBWG16,

    /// Sparse Color Filter #1 White-Green-White-Blue 8-bit.
    SCF1WGWB8,

    /// Sparse Color Filter #1 White-Green-White-Blue 10-bit unpacked.
    SCF1WGWB10,

    /// Sparse Color Filter #1 White-Green-White-Blue 10-bit packed.
    SCF1WGWB10p,

    /// Sparse Color Filter #1 White-Green-White-Blue 12-bit unpacked.
    SCF1WGWB12,

    /// Sparse Color Filter #1 White-Green-White-Blue 12-bit packed.
    SCF1WGWB12p,

    /// Sparse Color Filter #1 White-Green-White-Blue 14-bit unpacked.
    SCF1WGWB14,

    /// Sparse Color Filter #1 White-Green-White-Blue 16-bit.
    SCF1WGWB16,

    /// Sparse Color Filter #1 White-Green-White-Red 8-bit.
    SCF1WGWR8,

    /// Sparse Color Filter #1 White-Green-White-Red 10-bit unpacked.
    SCF1WGWR10,

    /// Sparse Color Filter #1 White-Green-White-Red 10-bit packed.
    SCF1WGWR10p,

    /// Sparse Color Filter #1 White-Green-White-Red 12-bit unpacked.
    SCF1WGWR12,

    /// Sparse Color Filter #1 White-Green-White-Red 12-bit packed.
    SCF1WGWR12p,

    /// Sparse Color Filter #1 White-Green-White-Red 14-bit unpacked.
    SCF1WGWR14,

    /// Sparse Color Filter #1 White-Green-White-Red 16-bit.
    SCF1WGWR16,

    /// Sparse Color Filter #1 White-Red-White-Green 8-bit.
    SCF1WRWG8,

    /// Sparse Color Filter #1 White-Red-White-Green 10-bit unpacked.
    SCF1WRWG10,

    /// Sparse Color Filter #1 White-Red-White-Green 10-bit packed.
    SCF1WRWG10p,

    /// Sparse Color Filter #1 White-Red-White-Green 12-bit unpacked.
    SCF1WRWG12,

    /// Sparse Color Filter #1 White-Red-White-Green 12-bit packed.
    SCF1WRWG12p,

    /// Sparse Color Filter #1 White-Red-White-Green 14-bit unpacked.
    SCF1WRWG14,

    /// Sparse Color Filter #1 White-Red-White-Green 16-bit.
    SCF1WRWG16,

    /// YCbCr 4:4:4 10-bit unpacked.
    YCbCr10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed.
    YCbCr10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked.
    YCbCr12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed.
    YCbCr12p_CbYCr,

    /// YCbCr 4:2:2 10-bit packed.
    YCbCr422_10p,

    /// YCbCr 4:2:2 12-bit packed.
    YCbCr422_12p,

    /// YCbCr 4:4:4 10-bit unpacked BT.601.
    YCbCr601_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.601.
    YCbCr601_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.601.
    YCbCr601_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.601.
    YCbCr601_12p_CbYCr,

    /// YCbCr 4:2:2 10-bit unpacked BT.601.
    YCbCr601_422_10,

    /// YCbCr 4:2:2 10-bit packed BT.601.
    YCbCr601_422_10p,

    /// YCbCr 4:2:2 12-bit unpacked BT.601.
    YCbCr601_422_12,

    /// YCbCr 4:2:2 12-bit packed BT.601.
    YCbCr601_422_12p,

    /// YCbCr 4:4:4 10-bit unpacked BT.709.
    YCbCr709_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.709.
    YCbCr709_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.709.
    YCbCr709_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.709.
    YCbCr709_12p_CbYCr,

    /// YCbCr 4:2:2 10-bit unpacked BT.709.
    YCbCr709_422_10,

    /// YCbCr 4:2:2 10-bit packed BT.709.
    YCbCr709_422_10p,

    /// YCbCr 4:2:2 12-bit unpacked BT.709.
    YCbCr709_422_12,

    /// YCbCr 4:2:2 12-bit packed BT.709.
    YCbCr709_422_12p,

    /// YCbCr 4:2:2 10-bit unpacked.
    YCbCr422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed.
    YCbCr422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked.
    YCbCr422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed.
    YCbCr422_12p_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.601.
    YCbCr601_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.601.
    YCbCr601_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.601.
    YCbCr601_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.601.
    YCbCr601_422_12p_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.709.
    YCbCr709_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.709.
    YCbCr709_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.709.
    YCbCr709_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.709.
    YCbCr709_422_12p_CbYCrY,

    /// Bi-color Red/Green - Blue/Green 8-bit.
    BiColorRGBG8,

    /// Bi-color Blue/Green - Red/Green 8-bit.
    BiColorBGRG8,

    /// Bi-color Red/Green - Blue/Green 10-bit unpacked.
    BiColorRGBG10,

    /// Bi-color Red/Green - Blue/Green 10-bit packed.
    BiColorRGBG10p,

    /// Bi-color Blue/Green - Red/Green 10-bit unpacked.
    BiColorBGRG10,

    /// Bi-color Blue/Green - Red/Green 10-bit packed.
    BiColorBGRG10p,

    /// Bi-color Red/Green - Blue/Green 12-bit unpacked.
    BiColorRGBG12,

    /// Bi-color Red/Green - Blue/Green 12-bit packed.
    BiColorRGBG12p,

    /// Bi-color Blue/Green - Red/Green 12-bit unpacked.
    BiColorBGRG12,

    /// Bi-color Blue/Green - Red/Green 12-bit packed.
    BiColorBGRG12p,

    /// 3D coordinate A 8-bit.
    Coord3D_A8,

    /// 3D coordinate B 8-bit.
    Coord3D_B8,

    /// 3D coordinate C 8-bit.
    Coord3D_C8,

    /// 3D coordinate A-B-C 8-bit.
    Coord3D_ABC8,

    /// 3D coordinate A-B-C 8-bit planar.
    Coord3D_ABC8_Planar,

    /// 3D coordinate A-C 8-bit.
    Coord3D_AC8,

    /// 3D coordinate A-C 8-bit planar.
    Coord3D_AC8_Planar,

    /// 3D coordinate A 16-bit.
    Coord3D_A16,

    /// 3D coordinate B 16-bit.
    Coord3D_B16,

    /// 3D coordinate C 16-bit.
    Coord3D_C16,

    /// 3D coordinate A-B-C 16-bit.
    Coord3D_ABC16,

    /// 3D coordinate A-B-C 16-bit planar.
    Coord3D_ABC16_Planar,

    /// 3D coordinate A-C 16-bit.
    Coord3D_AC16,

    /// 3D coordinate A-C 16-bit planar.
    Coord3D_AC16_Planar,

    /// 3D coordinate A 32-bit floating point.
    Coord3D_A32f,

    /// 3D coordinate B 32-bit floating point.
    Coord3D_B32f,

    /// 3D coordinate C 32-bit floating point.
    Coord3D_C32f,

    /// 3D coordinate A-B-C 32-bit floating point.
    Coord3D_ABC32f,

    /// 3D coordinate A-B-C 32-bit floating point planar.
    Coord3D_ABC32f_Planar,

    /// 3D coordinate A-C 32-bit floating point.
    Coord3D_AC32f,

    /// 3D coordinate A-C 32-bit floating point planar.
    Coord3D_AC32f_Planar,

    /// Confidence 1-bit unpacked.
    Confidence1,

    /// Confidence 1-bit packed.
    Confidence1p,

    /// Confidence 8-bit.
    Confidence8,

    /// Confidence 16-bit.
    Confidence16,

    /// Confidence 32-bit floating point.
    Confidence32f,

    /// Red 8-bit.
    R8,

    /// Red 10-bit.
    R10,

    /// Red 12-bit.
    R12,

    /// Red 16-bit.
    R16,

    /// Green 8-bit.
    G8,

    /// Green 10-bit.
    G10,

    /// Green 12-bit.
    G12,

    /// Green 16-bit.
    G16,

    /// Blue 8-bit.
    B8,

    /// Blue 10-bit.
    B10,

    /// Blue 12-bit.
    B12,

    /// Blue 16-bit.
    B16,

    /// 3D coordinate A 10-bit packed.
    Coord3D_A10p,

    /// 3D coordinate B 10-bit packed.
    Coord3D_B10p,

    /// 3D coordinate C 10-bit packed.
    Coord3D_C10p,

    /// 3D coordinate A 12-bit packed.
    Coord3D_A12p,

    /// 3D coordinate B 12-bit packed.
    Coord3D_B12p,

    /// 3D coordinate C 12-bit packed.
    Coord3D_C12p,

    /// 3D coordinate A-B-C 10-bit packed.
    Coord3D_ABC10p,

    /// 3D coordinate A-B-C 10-bit packed planar.
    Coord3D_ABC10p_Planar,

    /// 3D coordinate A-B-C 12-bit packed.
    Coord3D_ABC12p,

    /// 3D coordinate A-B-C 12-bit packed planar.
    Coord3D_ABC12p_Planar,

    /// 3D coordinate A-C 10-bit packed.
    Coord3D_AC10p,

    /// 3D coordinate A-C 10-bit packed planar.
    Coord3D_AC10p_Planar,

    /// 3D coordinate A-C 12-bit packed.
    Coord3D_AC12p,

    /// 3D coordinate A-C 12-bit packed planar.
    Coord3D_AC12p_Planar,

    /// YCbCr 4:4:4 8-bit BT.2020.
    YCbCr2020_8_CbYCr,

    /// YCbCr 4:4:4 10-bit unpacked BT.2020.
    YCbCr2020_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.2020.
    YCbCr2020_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.2020.
    YCbCr2020_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.2020.
    YCbCr2020_12p_CbYCr,

    /// YCbCr 4:1:1 8-bit BT.2020.
    YCbCr2020_411_8_CbYYCrYY,

    /// YCbCr 4:2:2 8-bit BT.2020.
    YCbCr2020_422_8,

    /// YCbCr 4:2:2 8-bit BT.2020.
    YCbCr2020_422_8_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.2020.
    YCbCr2020_422_10,

    /// YCbCr 4:2:2 10-bit unpacked BT.2020.
    YCbCr2020_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.2020.
    YCbCr2020_422_10p,

    /// YCbCr 4:2:2 10-bit packed BT.2020.
    YCbCr2020_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.2020.
    YCbCr2020_422_12,

    /// YCbCr 4:2:2 12-bit unpacked BT.2020.
    YCbCr2020_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.2020.
    YCbCr2020_422_12p,

    /// YCbCr 4:2:2 12-bit packed BT.2020.
    YCbCr2020_422_12p_CbYCrY,

    /// Monochrome 14-bit packed.
    Mono14p,

    /// Bayer Green-Red 14-bit packed.
    BayerGR14p,

    /// Bayer Red-Green 14-bit packed.
    BayerRG14p,

    /// Bayer Green-Blue 14-bit packed.
    BayerGB14p,

    /// Bayer Blue-Green 14-bit packed.
    BayerBG14p,

    /// Bayer Green-Red 14-bit.
    BayerGR14,

    /// Bayer Red-Green 14-bit.
    BayerRG14,

    /// Bayer Green-Blue 14-bit.
    BayerGB14,

    /// Bayer Blue-Green 14-bit.
    BayerBG14,

    /// Bayer Green-Red 4-bit packed.
    BayerGR4p,

    /// Bayer Red-Green 4-bit packed.
    BayerRG4p,

    /// Bayer Green-Blue 4-bit packed.
    BayerGB4p,

    /// Bayer Blue-Green 4-bit packed.
    BayerBG4p,

    /// Monochrome 32-bit.
    Mono32,

    /// YCbCr 4:2:0 8-bit YY/CbCr Semiplanar.
    YCbCr420_8_YY_CbCr_Semiplanar,

    /// YCbCr 4:2:2 8-bit YY/CbCr Semiplanar.
    YCbCr422_8_YY_CbCr_Semiplanar,

    /// YCbCr 4:2:0 8-bit YY/CrCb Semiplanar.
    YCbCr420_8_YY_CrCb_Semiplanar,

    /// YCbCr 4:2:2 8-bit YY/CrCb Semiplanar.
    YCbCr422_8_YY_CrCb_Semiplanar,

    /// Data 8-bit.
    Data8,

    /// Data 8-bit signed.
    Data8s,

    /// Data 16-bit.
    Data16,

    /// Data 16-bit signed.
    Data16s,

    /// Data 32-bit.
    Data32,

    /// Data 32-bit signed.
    Data32s,

    /// Data 32-bit floating point.
    Data32f,

    /// Data 64-bit.
    Data64,

    /// Data 64-bit signed.
    Data64s,

    /// Data 64-bit floating point.
    Data64f,
}

impl TryFrom<u32> for PixelFormat {
    type Error = String;

    #[allow(clippy::too_many_lines)]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x0108_0001 => Ok(Mono8),
            0x0108_0002 => Ok(Mono8s),
            0x0110_0003 => Ok(Mono10),
            0x010C_0004 => Ok(Mono10Packed),
            0x0110_0005 => Ok(Mono12),
            0x010C_0006 => Ok(Mono12Packed),
            0x0110_0007 => Ok(Mono16),
            0x0108_0008 => Ok(BayerGR8),
            0x0108_0009 => Ok(BayerRG8),
            0x0108_000A => Ok(BayerGB8),
            0x0108_000B => Ok(BayerBG8),
            0x0110_000C => Ok(BayerGR10),
            0x0110_000D => Ok(BayerRG10),
            0x0110_000E => Ok(BayerGB10),
            0x0110_000F => Ok(BayerBG10),
            0x0110_0010 => Ok(BayerGR12),
            0x0110_0011 => Ok(BayerRG12),
            0x0110_0012 => Ok(BayerGB12),
            0x0110_0013 => Ok(BayerBG12),
            0x0218_0014 => Ok(RGB8),
            0x0218_0015 => Ok(BGR8),
            0x0220_0016 => Ok(RGBa8),
            0x0220_0017 => Ok(BGRa8),
            0x0230_0018 => Ok(RGB10),
            0x0230_0019 => Ok(BGR10),
            0x0230_001A => Ok(RGB12),
            0x0230_001B => Ok(BGR12),
            0x0218_0020 => Ok(YUV8_UYV),
            0x0218_0021 => Ok(RGB8_Planar),
            0x0230_0022 => Ok(RGB10_Planar),
            0x0230_0023 => Ok(RGB12_Planar),
            0x0230_0024 => Ok(RGB16_Planar),
            0x0110_0025 => Ok(Mono14),
            0x010C_0026 => Ok(BayerGR10Packed),
            0x010C_0027 => Ok(BayerRG10Packed),
            0x010C_0028 => Ok(BayerGB10Packed),
            0x010C_0029 => Ok(BayerBG10Packed),
            0x010C_002A => Ok(BayerGR12Packed),
            0x010C_002B => Ok(BayerRG12Packed),
            0x010C_002C => Ok(BayerGB12Packed),
            0x010C_002D => Ok(BayerBG12Packed),
            0x0110_002E => Ok(BayerGR16),
            0x0110_002F => Ok(BayerRG16),
            0x0110_0030 => Ok(BayerGB16),
            0x0110_0031 => Ok(BayerBG16),
            0x0210_0032 => Ok(YUV422_8),
            0x0230_0033 => Ok(RGB16),
            0x0224_0034 => Ok(RGB12V1Packed),
            0x0210_0035 => Ok(RGB565p),
            0x0210_0036 => Ok(BGR565p),
            0x0101_0037 => Ok(Mono1p),
            0x0102_0038 => Ok(Mono2p),
            0x0104_0039 => Ok(Mono4p),
            0x0218_003A => Ok(YCbCr8_CbYCr),
            0x0210_003B => Ok(YCbCr422_8),
            0x020C_003C => Ok(YCbCr411_8_CbYYCrYY),
            0x0218_003D => Ok(YCbCr601_8_CbYCr),
            0x0210_003E => Ok(YCbCr601_422_8),
            0x020C_003F => Ok(YCbCr601_411_8_CbYYCrYY),
            0x0218_0040 => Ok(YCbCr709_8_CbYCr),
            0x0210_0041 => Ok(YCbCr709_422_8),
            0x020C_0042 => Ok(YCbCr709_411_8_CbYYCrYY),
            0x0210_0043 => Ok(YCbCr422_8_CbYCrY),
            0x0210_0044 => Ok(YCbCr601_422_8_CbYCrY),
            0x0210_0045 => Ok(YCbCr709_422_8_CbYCrY),
            0x010A_0046 => Ok(Mono10p),
            0x010C_0047 => Ok(Mono12p),
            0x021E_0048 => Ok(BGR10p),
            0x0224_0049 => Ok(BGR12p),
            0x0230_004A => Ok(BGR14),
            0x0230_004B => Ok(BGR16),
            0x0240_004C => Ok(BGRa10),
            0x0228_004D => Ok(BGRa10p),
            0x0240_004E => Ok(BGRa12),
            0x0230_004F => Ok(BGRa12p),
            0x0240_0050 => Ok(BGRa14),
            0x0240_0051 => Ok(BGRa16),
            0x010A_0052 => Ok(BayerBG10p),
            0x010C_0053 => Ok(BayerBG12p),
            0x010A_0054 => Ok(BayerGB10p),
            0x010C_0055 => Ok(BayerGB12p),
            0x010A_0056 => Ok(BayerGR10p),
            0x010C_0057 => Ok(BayerGR12p),
            0x010A_0058 => Ok(BayerRG10p),
            0x010C_0059 => Ok(BayerRG12p),
            0x020C_005A => Ok(YCbCr411_8),
            0x0218_005B => Ok(YCbCr8),
            0x021E_005C => Ok(RGB10p),
            0x0224_005D => Ok(RGB12p),
            0x0230_005E => Ok(RGB14),
            0x0240_005F => Ok(RGBa10),
            0x0228_0060 => Ok(RGBa10p),
            0x0240_0061 => Ok(RGBa12),
            0x0230_0062 => Ok(RGBa12p),
            0x0240_0063 => Ok(RGBa14),
            0x0240_0064 => Ok(RGBa16),
            0x0220_0065 => Ok(YCbCr422_10),
            0x0220_0066 => Ok(YCbCr422_12),
            0x0108_0067 => Ok(SCF1WBWG8),
            0x0110_0068 => Ok(SCF1WBWG10),
            0x010A_0069 => Ok(SCF1WBWG10p),
            0x0110_006A => Ok(SCF1WBWG12),
            0x010C_006B => Ok(SCF1WBWG12p),
            0x0110_006C => Ok(SCF1WBWG14),
            0x0110_006D => Ok(SCF1WBWG16),
            0x0108_006E => Ok(SCF1WGWB8),
            0x0110_006F => Ok(SCF1WGWB10),
            0x010A_0070 => Ok(SCF1WGWB10p),
            0x0110_0071 => Ok(SCF1WGWB12),
            0x010C_0072 => Ok(SCF1WGWB12p),
            0x0110_0073 => Ok(SCF1WGWB14),
            0x0110_0074 => Ok(SCF1WGWB16),
            0x0108_0075 => Ok(SCF1WGWR8),
            0x0110_0076 => Ok(SCF1WGWR10),
            0x010A_0077 => Ok(SCF1WGWR10p),
            0x0110_0078 => Ok(SCF1WGWR12),
            0x010C_0079 => Ok(SCF1WGWR12p),
            0x0110_007A => Ok(SCF1WGWR14),
            0x0110_007B => Ok(SCF1WGWR16),
            0x0108_007C => Ok(SCF1WRWG8),
            0x0110_007D => Ok(SCF1WRWG10),
            0x010A_007E => Ok(SCF1WRWG10p),
            0x0110_007F => Ok(SCF1WRWG12),
            0x010C_0080 => Ok(SCF1WRWG12p),
            0x0110_0081 => Ok(SCF1WRWG14),
            0x0110_0082 => Ok(SCF1WRWG16),
            0x0230_0083 => Ok(YCbCr10_CbYCr),
            0x021E_0084 => Ok(YCbCr10p_CbYCr),
            0x0230_0085 => Ok(YCbCr12_CbYCr),
            0x0224_0086 => Ok(YCbCr12p_CbYCr),
            0x0214_0087 => Ok(YCbCr422_10p),
            0x0218_0088 => Ok(YCbCr422_12p),
            0x0230_0089 => Ok(YCbCr601_10_CbYCr),
            0x021E_008A => Ok(YCbCr601_10p_CbYCr),
            0x0230_008B => Ok(YCbCr601_12_CbYCr),
            0x0224_008C => Ok(YCbCr601_12p_CbYCr),
            0x0220_008D => Ok(YCbCr601_422_10),
            0x0214_008E => Ok(YCbCr601_422_10p),
            0x0220_008F => Ok(YCbCr601_422_12),
            0x0218_0090 => Ok(YCbCr601_422_12p),
            0x0230_0091 => Ok(YCbCr709_10_CbYCr),
            0x021E_0092 => Ok(YCbCr709_10p_CbYCr),
            0x0230_0093 => Ok(YCbCr709_12_CbYCr),
            0x0224_0094 => Ok(YCbCr709_12p_CbYCr),
            0x0220_0095 => Ok(YCbCr709_422_10),
            0x0214_0096 => Ok(YCbCr709_422_10p),
            0x0220_0097 => Ok(YCbCr709_422_12),
            0x0218_0098 => Ok(YCbCr709_422_12p),
            0x0220_0099 => Ok(YCbCr422_10_CbYCrY),
            0x0214_009A => Ok(YCbCr422_10p_CbYCrY),
            0x0220_009B => Ok(YCbCr422_12_CbYCrY),
            0x0218_009C => Ok(YCbCr422_12p_CbYCrY),
            0x0220_009D => Ok(YCbCr601_422_10_CbYCrY),
            0x0214_009E => Ok(YCbCr601_422_10p_CbYCrY),
            0x0220_009F => Ok(YCbCr601_422_12_CbYCrY),
            0x0218_00A0 => Ok(YCbCr601_422_12p_CbYCrY),
            0x0220_00A1 => Ok(YCbCr709_422_10_CbYCrY),
            0x0214_00A2 => Ok(YCbCr709_422_10p_CbYCrY),
            0x0220_00A3 => Ok(YCbCr709_422_12_CbYCrY),
            0x0218_00A4 => Ok(YCbCr709_422_12p_CbYCrY),
            0x0210_00A5 => Ok(BiColorRGBG8),
            0x0210_00A6 => Ok(BiColorBGRG8),
            0x0220_00A7 => Ok(BiColorRGBG10),
            0x0214_00A8 => Ok(BiColorRGBG10p),
            0x0220_00A9 => Ok(BiColorBGRG10),
            0x0214_00AA => Ok(BiColorBGRG10p),
            0x0220_00AB => Ok(BiColorRGBG12),
            0x0218_00AC => Ok(BiColorRGBG12p),
            0x0220_00AD => Ok(BiColorBGRG12),
            0x0218_00AE => Ok(BiColorBGRG12p),
            0x0108_00AF => Ok(Coord3D_A8),
            0x0108_00B0 => Ok(Coord3D_B8),
            0x0108_00B1 => Ok(Coord3D_C8),
            0x0218_00B2 => Ok(Coord3D_ABC8),
            0x0218_00B3 => Ok(Coord3D_ABC8_Planar),
            0x0210_00B4 => Ok(Coord3D_AC8),
            0x0210_00B5 => Ok(Coord3D_AC8_Planar),
            0x0110_00B6 => Ok(Coord3D_A16),
            0x0110_00B7 => Ok(Coord3D_B16),
            0x0110_00B8 => Ok(Coord3D_C16),
            0x0230_00B9 => Ok(Coord3D_ABC16),
            0x0230_00BA => Ok(Coord3D_ABC16_Planar),
            0x0220_00BB => Ok(Coord3D_AC16),
            0x0220_00BC => Ok(Coord3D_AC16_Planar),
            0x0120_00BD => Ok(Coord3D_A32f),
            0x0120_00BE => Ok(Coord3D_B32f),
            0x0120_00BF => Ok(Coord3D_C32f),
            0x0260_00C0 => Ok(Coord3D_ABC32f),
            0x0260_00C1 => Ok(Coord3D_ABC32f_Planar),
            0x0240_00C2 => Ok(Coord3D_AC32f),
            0x0240_00C3 => Ok(Coord3D_AC32f_Planar),
            0x0108_00C4 => Ok(Confidence1),
            0x0101_00C5 => Ok(Confidence1p),
            0x0108_00C6 => Ok(Confidence8),
            0x0110_00C7 => Ok(Confidence16),
            0x0120_00C8 => Ok(Confidence32f),
            0x0108_00C9 => Ok(R8),
            0x010A_00CA => Ok(R10),
            0x010C_00CB => Ok(R12),
            0x0110_00CC => Ok(R16),
            0x0108_00CD => Ok(G8),
            0x010A_00CE => Ok(G10),
            0x010C_00CF => Ok(G12),
            0x0110_00D0 => Ok(G16),
            0x0108_00D1 => Ok(B8),
            0x010A_00D2 => Ok(B10),
            0x010C_00D3 => Ok(B12),
            0x0110_00D4 => Ok(B16),
            0x010A_00D5 => Ok(Coord3D_A10p),
            0x010A_00D6 => Ok(Coord3D_B10p),
            0x010A_00D7 => Ok(Coord3D_C10p),
            0x010C_00D8 => Ok(Coord3D_A12p),
            0x010C_00D9 => Ok(Coord3D_B12p),
            0x010C_00DA => Ok(Coord3D_C12p),
            0x021E_00DB => Ok(Coord3D_ABC10p),
            0x021E_00DC => Ok(Coord3D_ABC10p_Planar),
            0x0224_00DE => Ok(Coord3D_ABC12p),
            0x0224_00DF => Ok(Coord3D_ABC12p_Planar),
            0x0214_00F0 => Ok(Coord3D_AC10p),
            0x0214_00F1 => Ok(Coord3D_AC10p_Planar),
            0x0218_00F2 => Ok(Coord3D_AC12p),
            0x0218_00F3 => Ok(Coord3D_AC12p_Planar),
            0x0218_00F4 => Ok(YCbCr2020_8_CbYCr),
            0x0230_00F5 => Ok(YCbCr2020_10_CbYCr),
            0x021E_00F6 => Ok(YCbCr2020_10p_CbYCr),
            0x0230_00F7 => Ok(YCbCr2020_12_CbYCr),
            0x0224_00F8 => Ok(YCbCr2020_12p_CbYCr),
            0x020C_00F9 => Ok(YCbCr2020_411_8_CbYYCrYY),
            0x0210_00FA => Ok(YCbCr2020_422_8),
            0x0210_00FB => Ok(YCbCr2020_422_8_CbYCrY),
            0x0220_00FC => Ok(YCbCr2020_422_10),
            0x0220_00FD => Ok(YCbCr2020_422_10_CbYCrY),
            0x0214_00FE => Ok(YCbCr2020_422_10p),
            0x0214_00FF => Ok(YCbCr2020_422_10p_CbYCrY),
            0x0220_0100 => Ok(YCbCr2020_422_12),
            0x0220_0101 => Ok(YCbCr2020_422_12_CbYCrY),
            0x0218_0102 => Ok(YCbCr2020_422_12p),
            0x0218_0103 => Ok(YCbCr2020_422_12p_CbYCrY),
            0x010E_0104 => Ok(Mono14p),
            0x010E_0105 => Ok(BayerGR14p),
            0x010E_0106 => Ok(BayerRG14p),
            0x010E_0107 => Ok(BayerGB14p),
            0x010E_0108 => Ok(BayerBG14p),
            0x0110_0109 => Ok(BayerGR14),
            0x0110_010A => Ok(BayerRG14),
            0x0110_010B => Ok(BayerGB14),
            0x0110_010C => Ok(BayerBG14),
            0x0104_010D => Ok(BayerGR4p),
            0x0104_010E => Ok(BayerRG4p),
            0x0104_010F => Ok(BayerGB4p),
            0x0104_0110 => Ok(BayerBG4p),
            0x0120_0111 => Ok(Mono32),
            0x020C_0112 => Ok(YCbCr420_8_YY_CbCr_Semiplanar),
            0x0210_0113 => Ok(YCbCr422_8_YY_CbCr_Semiplanar),
            0x020C_0114 => Ok(YCbCr420_8_YY_CrCb_Semiplanar),
            0x0210_0115 => Ok(YCbCr422_8_YY_CrCb_Semiplanar),
            0x0108_0116 => Ok(Data8),
            0x0108_0117 => Ok(Data8s),
            0x0110_0118 => Ok(Data16),
            0x0110_0119 => Ok(Data16s),
            0x0120_011A => Ok(Data32),
            0x0120_011B => Ok(Data32s),
            0x0120_011C => Ok(Data32f),
            0x0140_011D => Ok(Data64),
            0x0140_011E => Ok(Data64s),
            0x0140_011F => Ok(Data64f),
            otherwise => Err(format!("{:x} is invalid value for pixel format", otherwise)),
        }
    }
}

impl Into<u32> for PixelFormat {
    #[allow(clippy::too_many_lines)]
    fn into(self) -> u32 {
        match self {
            Mono8 => 0x0108_0001,
            Mono8s => 0x0108_0002,
            Mono10 => 0x0110_0003,
            Mono10Packed => 0x010C_0004,
            Mono12 => 0x0110_0005,
            Mono12Packed => 0x010C_0006,
            Mono16 => 0x0110_0007,
            BayerGR8 => 0x0108_0008,
            BayerRG8 => 0x0108_0009,
            BayerGB8 => 0x0108_000A,
            BayerBG8 => 0x0108_000B,
            BayerGR10 => 0x0110_000C,
            BayerRG10 => 0x0110_000D,
            BayerGB10 => 0x0110_000E,
            BayerBG10 => 0x0110_000F,
            BayerGR12 => 0x0110_0010,
            BayerRG12 => 0x0110_0011,
            BayerGB12 => 0x0110_0012,
            BayerBG12 => 0x0110_0013,
            RGB8 => 0x0218_0014,
            BGR8 => 0x0218_0015,
            RGBa8 => 0x0220_0016,
            BGRa8 => 0x0220_0017,
            RGB10 => 0x0230_0018,
            BGR10 => 0x0230_0019,
            RGB12 => 0x0230_001A,
            BGR12 => 0x0230_001B,
            YUV8_UYV => 0x0218_0020,
            RGB8_Planar => 0x0218_0021,
            RGB10_Planar => 0x0230_0022,
            RGB12_Planar => 0x0230_0023,
            RGB16_Planar => 0x0230_0024,
            Mono14 => 0x0110_0025,
            BayerGR10Packed => 0x010C_0026,
            BayerRG10Packed => 0x010C_0027,
            BayerGB10Packed => 0x010C_0028,
            BayerBG10Packed => 0x010C_0029,
            BayerGR12Packed => 0x010C_002A,
            BayerRG12Packed => 0x010C_002B,
            BayerGB12Packed => 0x010C_002C,
            BayerBG12Packed => 0x010C_002D,
            BayerGR16 => 0x0110_002E,
            BayerRG16 => 0x0110_002F,
            BayerGB16 => 0x0110_0030,
            BayerBG16 => 0x0110_0031,
            YUV422_8 => 0x0210_0032,
            RGB16 => 0x0230_0033,
            RGB12V1Packed => 0x0224_0034,
            RGB565p => 0x0210_0035,
            BGR565p => 0x0210_0036,
            Mono1p => 0x0101_0037,
            Mono2p => 0x0102_0038,
            Mono4p => 0x0104_0039,
            YCbCr8_CbYCr => 0x0218_003A,
            YCbCr422_8 => 0x0210_003B,
            YCbCr411_8_CbYYCrYY => 0x020C_003C,
            YCbCr601_8_CbYCr => 0x0218_003D,
            YCbCr601_422_8 => 0x0210_003E,
            YCbCr601_411_8_CbYYCrYY => 0x020C_003F,
            YCbCr709_8_CbYCr => 0x0218_0040,
            YCbCr709_422_8 => 0x0210_0041,
            YCbCr709_411_8_CbYYCrYY => 0x020C_0042,
            YCbCr422_8_CbYCrY => 0x0210_0043,
            YCbCr601_422_8_CbYCrY => 0x0210_0044,
            YCbCr709_422_8_CbYCrY => 0x0210_0045,
            Mono10p => 0x010A_0046,
            Mono12p => 0x010C_0047,
            BGR10p => 0x021E_0048,
            BGR12p => 0x0224_0049,
            BGR14 => 0x0230_004A,
            BGR16 => 0x0230_004B,
            BGRa10 => 0x0240_004C,
            BGRa10p => 0x0228_004D,
            BGRa12 => 0x0240_004E,
            BGRa12p => 0x0230_004F,
            BGRa14 => 0x0240_0050,
            BGRa16 => 0x0240_0051,
            BayerBG10p => 0x010A_0052,
            BayerBG12p => 0x010C_0053,
            BayerGB10p => 0x010A_0054,
            BayerGB12p => 0x010C_0055,
            BayerGR10p => 0x010A_0056,
            BayerGR12p => 0x010C_0057,
            BayerRG10p => 0x010A_0058,
            BayerRG12p => 0x010C_0059,
            YCbCr411_8 => 0x020C_005A,
            YCbCr8 => 0x0218_005B,
            RGB10p => 0x021E_005C,
            RGB12p => 0x0224_005D,
            RGB14 => 0x0230_005E,
            RGBa10 => 0x0240_005F,
            RGBa10p => 0x0228_0060,
            RGBa12 => 0x0240_0061,
            RGBa12p => 0x0230_0062,
            RGBa14 => 0x0240_0063,
            RGBa16 => 0x0240_0064,
            YCbCr422_10 => 0x0220_0065,
            YCbCr422_12 => 0x0220_0066,
            SCF1WBWG8 => 0x0108_0067,
            SCF1WBWG10 => 0x0110_0068,
            SCF1WBWG10p => 0x010A_0069,
            SCF1WBWG12 => 0x0110_006A,
            SCF1WBWG12p => 0x010C_006B,
            SCF1WBWG14 => 0x0110_006C,
            SCF1WBWG16 => 0x0110_006D,
            SCF1WGWB8 => 0x0108_006E,
            SCF1WGWB10 => 0x0110_006F,
            SCF1WGWB10p => 0x010A_0070,
            SCF1WGWB12 => 0x0110_0071,
            SCF1WGWB12p => 0x010C_0072,
            SCF1WGWB14 => 0x0110_0073,
            SCF1WGWB16 => 0x0110_0074,
            SCF1WGWR8 => 0x0108_0075,
            SCF1WGWR10 => 0x0110_0076,
            SCF1WGWR10p => 0x010A_0077,
            SCF1WGWR12 => 0x0110_0078,
            SCF1WGWR12p => 0x010C_0079,
            SCF1WGWR14 => 0x0110_007A,
            SCF1WGWR16 => 0x0110_007B,
            SCF1WRWG8 => 0x0108_007C,
            SCF1WRWG10 => 0x0110_007D,
            SCF1WRWG10p => 0x010A_007E,
            SCF1WRWG12 => 0x0110_007F,
            SCF1WRWG12p => 0x010C_0080,
            SCF1WRWG14 => 0x0110_0081,
            SCF1WRWG16 => 0x0110_0082,
            YCbCr10_CbYCr => 0x0230_0083,
            YCbCr10p_CbYCr => 0x021E_0084,
            YCbCr12_CbYCr => 0x0230_0085,
            YCbCr12p_CbYCr => 0x0224_0086,
            YCbCr422_10p => 0x0214_0087,
            YCbCr422_12p => 0x0218_0088,
            YCbCr601_10_CbYCr => 0x0230_0089,
            YCbCr601_10p_CbYCr => 0x021E_008A,
            YCbCr601_12_CbYCr => 0x0230_008B,
            YCbCr601_12p_CbYCr => 0x0224_008C,
            YCbCr601_422_10 => 0x0220_008D,
            YCbCr601_422_10p => 0x0214_008E,
            YCbCr601_422_12 => 0x0220_008F,
            YCbCr601_422_12p => 0x0218_0090,
            YCbCr709_10_CbYCr => 0x0230_0091,
            YCbCr709_10p_CbYCr => 0x021E_0092,
            YCbCr709_12_CbYCr => 0x0230_0093,
            YCbCr709_12p_CbYCr => 0x0224_0094,
            YCbCr709_422_10 => 0x0220_0095,
            YCbCr709_422_10p => 0x0214_0096,
            YCbCr709_422_12 => 0x0220_0097,
            YCbCr709_422_12p => 0x0218_0098,
            YCbCr422_10_CbYCrY => 0x0220_0099,
            YCbCr422_10p_CbYCrY => 0x0214_009A,
            YCbCr422_12_CbYCrY => 0x0220_009B,
            YCbCr422_12p_CbYCrY => 0x0218_009C,
            YCbCr601_422_10_CbYCrY => 0x0220_009D,
            YCbCr601_422_10p_CbYCrY => 0x0214_009E,
            YCbCr601_422_12_CbYCrY => 0x0220_009F,
            YCbCr601_422_12p_CbYCrY => 0x0218_00A0,
            YCbCr709_422_10_CbYCrY => 0x0220_00A1,
            YCbCr709_422_10p_CbYCrY => 0x0214_00A2,
            YCbCr709_422_12_CbYCrY => 0x0220_00A3,
            YCbCr709_422_12p_CbYCrY => 0x0218_00A4,
            BiColorRGBG8 => 0x0210_00A5,
            BiColorBGRG8 => 0x0210_00A6,
            BiColorRGBG10 => 0x0220_00A7,
            BiColorRGBG10p => 0x0214_00A8,
            BiColorBGRG10 => 0x0220_00A9,
            BiColorBGRG10p => 0x0214_00AA,
            BiColorRGBG12 => 0x0220_00AB,
            BiColorRGBG12p => 0x0218_00AC,
            BiColorBGRG12 => 0x0220_00AD,
            BiColorBGRG12p => 0x0218_00AE,
            Coord3D_A8 => 0x0108_00AF,
            Coord3D_B8 => 0x0108_00B0,
            Coord3D_C8 => 0x0108_00B1,
            Coord3D_ABC8 => 0x0218_00B2,
            Coord3D_ABC8_Planar => 0x0218_00B3,
            Coord3D_AC8 => 0x0210_00B4,
            Coord3D_AC8_Planar => 0x0210_00B5,
            Coord3D_A16 => 0x0110_00B6,
            Coord3D_B16 => 0x0110_00B7,
            Coord3D_C16 => 0x0110_00B8,
            Coord3D_ABC16 => 0x0230_00B9,
            Coord3D_ABC16_Planar => 0x0230_00BA,
            Coord3D_AC16 => 0x0220_00BB,
            Coord3D_AC16_Planar => 0x0220_00BC,
            Coord3D_A32f => 0x0120_00BD,
            Coord3D_B32f => 0x0120_00BE,
            Coord3D_C32f => 0x0120_00BF,
            Coord3D_ABC32f => 0x0260_00C0,
            Coord3D_ABC32f_Planar => 0x0260_00C1,
            Coord3D_AC32f => 0x0240_00C2,
            Coord3D_AC32f_Planar => 0x0240_00C3,
            Confidence1 => 0x0108_00C4,
            Confidence1p => 0x0101_00C5,
            Confidence8 => 0x0108_00C6,
            Confidence16 => 0x0110_00C7,
            Confidence32f => 0x0120_00C8,
            R8 => 0x0108_00C9,
            R10 => 0x010A_00CA,
            R12 => 0x010C_00CB,
            R16 => 0x0110_00CC,
            G8 => 0x0108_00CD,
            G10 => 0x010A_00CE,
            G12 => 0x010C_00CF,
            G16 => 0x0110_00D0,
            B8 => 0x0108_00D1,
            B10 => 0x010A_00D2,
            B12 => 0x010C_00D3,
            B16 => 0x0110_00D4,
            Coord3D_A10p => 0x010A_00D5,
            Coord3D_B10p => 0x010A_00D6,
            Coord3D_C10p => 0x010A_00D7,
            Coord3D_A12p => 0x010C_00D8,
            Coord3D_B12p => 0x010C_00D9,
            Coord3D_C12p => 0x010C_00DA,
            Coord3D_ABC10p => 0x021E_00DB,
            Coord3D_ABC10p_Planar => 0x021E_00DC,
            Coord3D_ABC12p => 0x0224_00DE,
            Coord3D_ABC12p_Planar => 0x0224_00DF,
            Coord3D_AC10p => 0x0214_00F0,
            Coord3D_AC10p_Planar => 0x0214_00F1,
            Coord3D_AC12p => 0x0218_00F2,
            Coord3D_AC12p_Planar => 0x0218_00F3,
            YCbCr2020_8_CbYCr => 0x0218_00F4,
            YCbCr2020_10_CbYCr => 0x0230_00F5,
            YCbCr2020_10p_CbYCr => 0x021E_00F6,
            YCbCr2020_12_CbYCr => 0x0230_00F7,
            YCbCr2020_12p_CbYCr => 0x0224_00F8,
            YCbCr2020_411_8_CbYYCrYY => 0x020C_00F9,
            YCbCr2020_422_8 => 0x0210_00FA,
            YCbCr2020_422_8_CbYCrY => 0x0210_00FB,
            YCbCr2020_422_10 => 0x0220_00FC,
            YCbCr2020_422_10_CbYCrY => 0x0220_00FD,
            YCbCr2020_422_10p => 0x0214_00FE,
            YCbCr2020_422_10p_CbYCrY => 0x0214_00FF,
            YCbCr2020_422_12 => 0x0220_0100,
            YCbCr2020_422_12_CbYCrY => 0x0220_0101,
            YCbCr2020_422_12p => 0x0218_0102,
            YCbCr2020_422_12p_CbYCrY => 0x0218_0103,
            Mono14p => 0x010E_0104,
            BayerGR14p => 0x010E_0105,
            BayerRG14p => 0x010E_0106,
            BayerGB14p => 0x010E_0107,
            BayerBG14p => 0x010E_0108,
            BayerGR14 => 0x0110_0109,
            BayerRG14 => 0x0110_010A,
            BayerGB14 => 0x0110_010B,
            BayerBG14 => 0x0110_010C,
            BayerGR4p => 0x0104_010D,
            BayerRG4p => 0x0104_010E,
            BayerGB4p => 0x0104_010F,
            BayerBG4p => 0x0104_0110,
            Mono32 => 0x0120_0111,
            YCbCr420_8_YY_CbCr_Semiplanar => 0x020C_0112,
            YCbCr422_8_YY_CbCr_Semiplanar => 0x0210_0113,
            YCbCr420_8_YY_CrCb_Semiplanar => 0x020C_0114,
            YCbCr422_8_YY_CrCb_Semiplanar => 0x0210_0115,
            Data8 => 0x0108_0116,
            Data8s => 0x0108_0117,
            Data16 => 0x0110_0118,
            Data16s => 0x0110_0119,
            Data32 => 0x0120_011A,
            Data32s => 0x0120_011B,
            Data32f => 0x0120_011C,
            Data64 => 0x0140_011D,
            Data64s => 0x0140_011E,
            Data64f => 0x0140_011F,
        }
    }
}
