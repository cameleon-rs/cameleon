use std::convert::TryFrom;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Monochrome 8-bit
    Mono8,

    /// Monochrome 8-bit signed
    Mono8s,

    /// Monochrome 10-bit unpacked
    Mono10,

    /// Monochrome 10-bit packed
    Mono10Packed,

    /// Monochrome 12-bit unpacked
    Mono12,

    /// Monochrome 12-bit packed
    Mono12Packed,

    /// Monochrome 16-bit
    Mono16,

    /// Bayer Green-Red 8-bit
    BayerGR8,

    /// Bayer Red-Green 8-bit
    BayerRG8,

    /// Bayer Green-Blue 8-bit
    BayerGB8,

    /// Bayer Blue-Green 8-bit
    BayerBG8,

    /// Bayer Green-Red 10-bit unpacked
    BayerGR10,

    /// Bayer Red-Green 10-bit unpacked
    BayerRG10,

    /// Bayer Green-Blue 10-bit unpacked
    BayerGB10,

    /// Bayer Blue-Green 10-bit unpacked
    BayerBG10,

    /// Bayer Green-Red 12-bit unpacked
    BayerGR12,

    /// Bayer Red-Green 12-bit unpacked
    BayerRG12,

    /// Bayer Green-Blue 12-bit unpacked
    BayerGB12,

    /// Bayer Blue-Green 12-bit unpacked
    BayerBG12,

    /// Red-Green-Blue 8-bit
    RGB8,

    /// Blue-Green-Red 8-bit
    BGR8,

    /// Red-Green-Blue-alpha 8-bit
    RGBa8,

    /// Blue-Green-Red-alpha 8-bit
    BGRa8,

    /// Red-Green-Blue 10-bit unpacked
    RGB10,

    /// Blue-Green-Red 10-bit unpacked
    BGR10,

    /// Red-Green-Blue 12-bit unpacked
    RGB12,

    /// Blue-Green-Red 12-bit unpacked
    BGR12,

    /// YUV 4:4:4 8-bit
    YUV8_UYV,

    /// Red-Green-Blue 8-bit planar
    RGB8_Planar,

    /// Red-Green-Blue 10-bit unpacked planar
    RGB10_Planar,

    /// Red-Green-Blue 12-bit unpacked planar
    RGB12_Planar,

    /// Red-Green-Blue 16-bit planar
    RGB16_Planar,

    /// Monochrome 14-bit unpacked
    Mono14,

    /// Bayer Green-Red 10-bit packed
    BayerGR10Packed,

    /// Bayer Red-Green 10-bit packed
    BayerRG10Packed,

    /// Bayer Green-Blue 10-bit packed
    BayerGB10Packed,

    /// Bayer Blue-Green 10-bit packed
    BayerBG10Packed,

    /// Bayer Green-Red 12-bit packed
    BayerGR12Packed,

    /// Bayer Red-Green 12-bit packed
    BayerRG12Packed,

    /// Bayer Green-Blue 12-bit packed
    BayerGB12Packed,

    /// Bayer Blue-Green 12-bit packed
    BayerBG12Packed,

    /// Bayer Green-Red 16-bit
    BayerGR16,

    /// Bayer Red-Green 16-bit
    BayerRG16,

    /// Bayer Green-Blue 16-bit
    BayerGB16,

    /// Bayer Blue-Green 16-bit
    BayerBG16,

    /// YUV 4:2:2 8-bit
    YUV422_8,

    /// Red-Green-Blue 16-bit
    RGB16,

    /// Red-Green-Blue 12-bit packed - variant 1
    RGB12V1Packed,

    /// Red-Green-Blue 5/6/5-bit packed
    RGB565p,

    /// Blue-Green-Red 5/6/5-bit packed
    BGR565p,

    /// Monochrome 1-bit packed
    Mono1p,

    /// Monochrome 2-bit packed
    Mono2p,

    /// Monochrome 4-bit packed
    Mono4p,

    /// YCbCr 4:4:4 8-bit
    YCbCr8_CbYCr,

    /// YCbCr 4:2:2 8-bit
    YCbCr422_8,

    /// YCbCr 4:1:1 8-bit
    YCbCr411_8_CbYYCrYY,

    /// YCbCr 4:4:4 8-bit BT.601
    YCbCr601_8_CbYCr,

    /// YCbCr 4:2:2 8-bit BT.601
    YCbCr601_422_8,

    /// YCbCr 4:1:1 8-bit BT.601
    YCbCr601_411_8_CbYYCrYY,

    /// YCbCr 4:4:4 8-bit BT.709
    YCbCr709_8_CbYCr,

    /// YCbCr 4:2:2 8-bit BT.709
    YCbCr709_422_8,

    /// YCbCr 4:1:1 8-bit BT.709
    YCbCr709_411_8_CbYYCrYY,

    /// YCbCr 4:2:2 8-bit
    YCbCr422_8_CbYCrY,

    /// YCbCr 4:2:2 8-bit BT.601
    YCbCr601_422_8_CbYCrY,

    /// YCbCr 4:2:2 8-bit BT.709
    YCbCr709_422_8_CbYCrY,

    /// Monochrome 10-bit packed
    Mono10p,

    /// Monochrome 12-bit packed
    Mono12p,

    /// Blue-Green-Red 10-bit packed
    BGR10p,

    /// Blue-Green-Red 12-bit packed
    BGR12p,

    /// Blue-Green-Red 14-bit unpacked
    BGR14,

    /// Blue-Green-Red 16-bit
    BGR16,

    /// Blue-Green-Red-alpha 10-bit unpacked
    BGRa10,

    /// Blue-Green-Red-alpha 10-bit packed
    BGRa10p,

    /// Blue-Green-Red-alpha 12-bit unpacked
    BGRa12,

    /// Blue-Green-Red-alpha 12-bit packed
    BGRa12p,

    /// Blue-Green-Red-alpha 14-bit unpacked
    BGRa14,

    /// Blue-Green-Red-alpha 16-bit
    BGRa16,

    /// Bayer Blue-Green 10-bit packed
    BayerBG10p,

    /// Bayer Blue-Green 12-bit packed
    BayerBG12p,

    /// Bayer Green-Blue 10-bit packed
    BayerGB10p,

    /// Bayer Green-Blue 12-bit packed
    BayerGB12p,

    /// Bayer Green-Red 10-bit packed
    BayerGR10p,

    /// Bayer Green-Red 12-bit packed
    BayerGR12p,

    /// Bayer Red-Green 10-bit packed
    BayerRG10p,

    /// Bayer Red-Green 12-bit packed
    BayerRG12p,

    /// YCbCr 4:1:1 8-bit
    YCbCr411_8,

    /// YCbCr 4:4:4 8-bit
    YCbCr8,

    /// Red-Green-Blue 10-bit packed
    RGB10p,

    /// Red-Green-Blue 12-bit packed
    RGB12p,

    /// Red-Green-Blue 14-bit unpacked
    RGB14,

    /// Red-Green-Blue-alpha 10-bit unpacked
    RGBa10,

    /// Red-Green-Blue-alpha 10-bit packed
    RGBa10p,

    /// Red-Green-Blue-alpha 12-bit unpacked
    RGBa12,

    /// Red-Green-Blue-alpha 12-bit packed
    RGBa12p,

    /// Red-Green-Blue-alpha 14-bit unpacked
    RGBa14,

    /// Red-Green-Blue-alpha 16-bit
    RGBa16,

    /// YCbCr 4:2:2 10-bit unpacked
    YCbCr422_10,

    /// YCbCr 4:2:2 12-bit unpacked
    YCbCr422_12,

    /// Sparse Color Filter #1 White-Blue-White-Green 8-bit
    SCF1WBWG8,

    /// Sparse Color Filter #1 White-Blue-White-Green 10-bit unpacked
    SCF1WBWG10,

    /// Sparse Color Filter #1 White-Blue-White-Green 10-bit packed
    SCF1WBWG10p,

    /// Sparse Color Filter #1 White-Blue-White-Green 12-bit unpacked
    SCF1WBWG12,

    /// Sparse Color Filter #1 White-Blue-White-Green 12-bit packed
    SCF1WBWG12p,

    /// Sparse Color Filter #1 White-Blue-White-Green 14-bit unpacked
    SCF1WBWG14,

    /// Sparse Color Filter #1 White-Blue-White-Green 16-bit unpacked
    SCF1WBWG16,

    /// Sparse Color Filter #1 White-Green-White-Blue 8-bit
    SCF1WGWB8,

    /// Sparse Color Filter #1 White-Green-White-Blue 10-bit unpacked
    SCF1WGWB10,

    /// Sparse Color Filter #1 White-Green-White-Blue 10-bit packed
    SCF1WGWB10p,

    /// Sparse Color Filter #1 White-Green-White-Blue 12-bit unpacked
    SCF1WGWB12,

    /// Sparse Color Filter #1 White-Green-White-Blue 12-bit packed
    SCF1WGWB12p,

    /// Sparse Color Filter #1 White-Green-White-Blue 14-bit unpacked
    SCF1WGWB14,

    /// Sparse Color Filter #1 White-Green-White-Blue 16-bit
    SCF1WGWB16,

    /// Sparse Color Filter #1 White-Green-White-Red 8-bit
    SCF1WGWR8,

    /// Sparse Color Filter #1 White-Green-White-Red 10-bit unpacked
    SCF1WGWR10,

    /// Sparse Color Filter #1 White-Green-White-Red 10-bit packed
    SCF1WGWR10p,

    /// Sparse Color Filter #1 White-Green-White-Red 12-bit unpacked
    SCF1WGWR12,

    /// Sparse Color Filter #1 White-Green-White-Red 12-bit packed
    SCF1WGWR12p,

    /// Sparse Color Filter #1 White-Green-White-Red 14-bit unpacked
    SCF1WGWR14,

    /// Sparse Color Filter #1 White-Green-White-Red 16-bit
    SCF1WGWR16,

    /// Sparse Color Filter #1 White-Red-White-Green 8-bit
    SCF1WRWG8,

    /// Sparse Color Filter #1 White-Red-White-Green 10-bit unpacked
    SCF1WRWG10,

    /// Sparse Color Filter #1 White-Red-White-Green 10-bit packed
    SCF1WRWG10p,

    /// Sparse Color Filter #1 White-Red-White-Green 12-bit unpacked
    SCF1WRWG12,

    /// Sparse Color Filter #1 White-Red-White-Green 12-bit packed
    SCF1WRWG12p,

    /// Sparse Color Filter #1 White-Red-White-Green 14-bit unpacked
    SCF1WRWG14,

    /// Sparse Color Filter #1 White-Red-White-Green 16-bit
    SCF1WRWG16,

    /// YCbCr 4:4:4 10-bit unpacked
    YCbCr10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed
    YCbCr10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked
    YCbCr12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed
    YCbCr12p_CbYCr,

    /// YCbCr 4:2:2 10-bit packed
    YCbCr422_10p,

    /// YCbCr 4:2:2 12-bit packed
    YCbCr422_12p,

    /// YCbCr 4:4:4 10-bit unpacked BT.601
    YCbCr601_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.601
    YCbCr601_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.601
    YCbCr601_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.601
    YCbCr601_12p_CbYCr,

    /// YCbCr 4:2:2 10-bit unpacked BT.601
    YCbCr601_422_10,

    /// YCbCr 4:2:2 10-bit packed BT.601
    YCbCr601_422_10p,

    /// YCbCr 4:2:2 12-bit unpacked BT.601
    YCbCr601_422_12,

    /// YCbCr 4:2:2 12-bit packed BT.601
    YCbCr601_422_12p,

    /// YCbCr 4:4:4 10-bit unpacked BT.709
    YCbCr709_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.709
    YCbCr709_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.709
    YCbCr709_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.709
    YCbCr709_12p_CbYCr,

    /// YCbCr 4:2:2 10-bit unpacked BT.709
    YCbCr709_422_10,

    /// YCbCr 4:2:2 10-bit packed BT.709
    YCbCr709_422_10p,

    /// YCbCr 4:2:2 12-bit unpacked BT.709
    YCbCr709_422_12,

    /// YCbCr 4:2:2 12-bit packed BT.709
    YCbCr709_422_12p,

    /// YCbCr 4:2:2 10-bit unpacked
    YCbCr422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed
    YCbCr422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked
    YCbCr422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed
    YCbCr422_12p_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.601
    YCbCr601_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.601
    YCbCr601_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.601
    YCbCr601_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.601
    YCbCr601_422_12p_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.709
    YCbCr709_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.709
    YCbCr709_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.709
    YCbCr709_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.709
    YCbCr709_422_12p_CbYCrY,

    /// Bi-color Red/Green - Blue/Green 8-bit
    BiColorRGBG8,

    /// Bi-color Blue/Green - Red/Green 8-bit
    BiColorBGRG8,

    /// Bi-color Red/Green - Blue/Green 10-bit unpacked
    BiColorRGBG10,

    /// Bi-color Red/Green - Blue/Green 10-bit packed
    BiColorRGBG10p,

    /// Bi-color Blue/Green - Red/Green 10-bit unpacked
    BiColorBGRG10,

    /// Bi-color Blue/Green - Red/Green 10-bit packed
    BiColorBGRG10p,

    /// Bi-color Red/Green - Blue/Green 12-bit unpacked
    BiColorRGBG12,

    /// Bi-color Red/Green - Blue/Green 12-bit packed
    BiColorRGBG12p,

    /// Bi-color Blue/Green - Red/Green 12-bit unpacked
    BiColorBGRG12,

    /// Bi-color Blue/Green - Red/Green 12-bit packed
    BiColorBGRG12p,

    /// 3D coordinate A 8-bit
    Coord3D_A8,

    /// 3D coordinate B 8-bit
    Coord3D_B8,

    /// 3D coordinate C 8-bit
    Coord3D_C8,

    /// 3D coordinate A-B-C 8-bit
    Coord3D_ABC8,

    /// 3D coordinate A-B-C 8-bit planar
    Coord3D_ABC8_Planar,

    /// 3D coordinate A-C 8-bit
    Coord3D_AC8,

    /// 3D coordinate A-C 8-bit planar
    Coord3D_AC8_Planar,

    /// 3D coordinate A 16-bit
    Coord3D_A16,

    /// 3D coordinate B 16-bit
    Coord3D_B16,

    /// 3D coordinate C 16-bit
    Coord3D_C16,

    /// 3D coordinate A-B-C 16-bit
    Coord3D_ABC16,

    /// 3D coordinate A-B-C 16-bit planar
    Coord3D_ABC16_Planar,

    /// 3D coordinate A-C 16-bit
    Coord3D_AC16,

    /// 3D coordinate A-C 16-bit planar
    Coord3D_AC16_Planar,

    /// 3D coordinate A 32-bit floating point
    Coord3D_A32f,

    /// 3D coordinate B 32-bit floating point
    Coord3D_B32f,

    /// 3D coordinate C 32-bit floating point
    Coord3D_C32f,

    /// 3D coordinate A-B-C 32-bit floating point
    Coord3D_ABC32f,

    /// 3D coordinate A-B-C 32-bit floating point planar
    Coord3D_ABC32f_Planar,

    /// 3D coordinate A-C 32-bit floating point
    Coord3D_AC32f,

    /// 3D coordinate A-C 32-bit floating point planar
    Coord3D_AC32f_Planar,

    /// Confidence 1-bit unpacked
    Confidence1,

    /// Confidence 1-bit packed
    Confidence1p,

    /// Confidence 8-bit
    Confidence8,

    /// Confidence 16-bit
    Confidence16,

    /// Confidence 32-bit floating point
    Confidence32f,

    /// Red 8-bit
    R8,

    /// Red 10-bit
    R10,

    /// Red 12-bit
    R12,

    /// Red 16-bit
    R16,

    /// Green 8-bit
    G8,

    /// Green 10-bit
    G10,

    /// Green 12-bit
    G12,

    /// Green 16-bit
    G16,

    /// Blue 8-bit
    B8,

    /// Blue 10-bit
    B10,

    /// Blue 12-bit
    B12,

    /// Blue 16-bit
    B16,

    /// 3D coordinate A 10-bit packed
    Coord3D_A10p,

    /// 3D coordinate B 10-bit packed
    Coord3D_B10p,

    /// 3D coordinate C 10-bit packed
    Coord3D_C10p,

    /// 3D coordinate A 12-bit packed
    Coord3D_A12p,

    /// 3D coordinate B 12-bit packed
    Coord3D_B12p,

    /// 3D coordinate C 12-bit packed
    Coord3D_C12p,

    /// 3D coordinate A-B-C 10-bit packed
    Coord3D_ABC10p,

    /// 3D coordinate A-B-C 10-bit packed planar
    Coord3D_ABC10p_Planar,

    /// 3D coordinate A-B-C 12-bit packed
    Coord3D_ABC12p,

    /// 3D coordinate A-B-C 12-bit packed planar
    Coord3D_ABC12p_Planar,

    /// 3D coordinate A-C 10-bit packed
    Coord3D_AC10p,

    /// 3D coordinate A-C 10-bit packed planar
    Coord3D_AC10p_Planar,

    /// 3D coordinate A-C 12-bit packed
    Coord3D_AC12p,

    /// 3D coordinate A-C 12-bit packed planar
    Coord3D_AC12p_Planar,

    /// YCbCr 4:4:4 8-bit BT.2020
    YCbCr2020_8_CbYCr,

    /// YCbCr 4:4:4 10-bit unpacked BT.2020
    YCbCr2020_10_CbYCr,

    /// YCbCr 4:4:4 10-bit packed BT.2020
    YCbCr2020_10p_CbYCr,

    /// YCbCr 4:4:4 12-bit unpacked BT.2020
    YCbCr2020_12_CbYCr,

    /// YCbCr 4:4:4 12-bit packed BT.2020
    YCbCr2020_12p_CbYCr,

    /// YCbCr 4:1:1 8-bit BT.2020
    YCbCr2020_411_8_CbYYCrYY,

    /// YCbCr 4:2:2 8-bit BT.2020
    YCbCr2020_422_8,

    /// YCbCr 4:2:2 8-bit BT.2020
    YCbCr2020_422_8_CbYCrY,

    /// YCbCr 4:2:2 10-bit unpacked BT.2020
    YCbCr2020_422_10,

    /// YCbCr 4:2:2 10-bit unpacked BT.2020
    YCbCr2020_422_10_CbYCrY,

    /// YCbCr 4:2:2 10-bit packed BT.2020
    YCbCr2020_422_10p,

    /// YCbCr 4:2:2 10-bit packed BT.2020
    YCbCr2020_422_10p_CbYCrY,

    /// YCbCr 4:2:2 12-bit unpacked BT.2020
    YCbCr2020_422_12,

    /// YCbCr 4:2:2 12-bit unpacked BT.2020
    YCbCr2020_422_12_CbYCrY,

    /// YCbCr 4:2:2 12-bit packed BT.2020
    YCbCr2020_422_12p,

    /// YCbCr 4:2:2 12-bit packed BT.2020
    YCbCr2020_422_12p_CbYCrY,

    /// Monochrome 14-bit packed
    Mono14p,

    /// Bayer Green-Red 14-bit packed
    BayerGR14p,

    /// Bayer Red-Green 14-bit packed
    BayerRG14p,

    /// Bayer Green-Blue 14-bit packed
    BayerGB14p,

    /// Bayer Blue-Green 14-bit packed
    BayerBG14p,

    /// Bayer Green-Red 14-bit
    BayerGR14,

    /// Bayer Red-Green 14-bit
    BayerRG14,

    /// Bayer Green-Blue 14-bit
    BayerGB14,

    /// Bayer Blue-Green 14-bit
    BayerBG14,

    /// Bayer Green-Red 4-bit packed
    BayerGR4p,

    /// Bayer Red-Green 4-bit packed
    BayerRG4p,

    /// Bayer Green-Blue 4-bit packed
    BayerGB4p,

    /// Bayer Blue-Green 4-bit packed
    BayerBG4p,

    /// Monochrome 32-bit
    Mono32,

    /// YCbCr 4:2:0 8-bit YY/CbCr Semiplanar
    YCbCr420_8_YY_CbCr_Semiplanar,

    /// YCbCr 4:2:2 8-bit YY/CbCr Semiplanar
    YCbCr422_8_YY_CbCr_Semiplanar,

    /// YCbCr 4:2:0 8-bit YY/CrCb Semiplanar
    YCbCr420_8_YY_CrCb_Semiplanar,

    /// YCbCr 4:2:2 8-bit YY/CrCb Semiplanar
    YCbCr422_8_YY_CrCb_Semiplanar,

    /// Data 8-bit
    Data8,

    /// Data 8-bit signed
    Data8s,

    /// Data 16-bit
    Data16,

    /// Data 16-bit signed
    Data16s,

    /// Data 32-bit
    Data32,

    /// Data 32-bit signed
    Data32s,

    /// Data 32-bit floating point
    Data32f,

    /// Data 64-bit
    Data64,

    /// Data 64-bit signed
    Data64s,

    /// Data 64-bit floating point
    Data64f,
}

impl TryFrom<u32> for PixelFormat {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use PixelFormat::*;
        match value {
            0x01080001 => Ok(Mono8),
            0x01080002 => Ok(Mono8s),
            0x01100003 => Ok(Mono10),
            0x010C0004 => Ok(Mono10Packed),
            0x01100005 => Ok(Mono12),
            0x010C0006 => Ok(Mono12Packed),
            0x01100007 => Ok(Mono16),
            0x01080008 => Ok(BayerGR8),
            0x01080009 => Ok(BayerRG8),
            0x0108000A => Ok(BayerGB8),
            0x0108000B => Ok(BayerBG8),
            0x0110000C => Ok(BayerGR10),
            0x0110000D => Ok(BayerRG10),
            0x0110000E => Ok(BayerGB10),
            0x0110000F => Ok(BayerBG10),
            0x01100010 => Ok(BayerGR12),
            0x01100011 => Ok(BayerRG12),
            0x01100012 => Ok(BayerGB12),
            0x01100013 => Ok(BayerBG12),
            0x02180014 => Ok(RGB8),
            0x02180015 => Ok(BGR8),
            0x02200016 => Ok(RGBa8),
            0x02200017 => Ok(BGRa8),
            0x02300018 => Ok(RGB10),
            0x02300019 => Ok(BGR10),
            0x0230001A => Ok(RGB12),
            0x0230001B => Ok(BGR12),
            0x02180020 => Ok(YUV8_UYV),
            0x02180021 => Ok(RGB8_Planar),
            0x02300022 => Ok(RGB10_Planar),
            0x02300023 => Ok(RGB12_Planar),
            0x02300024 => Ok(RGB16_Planar),
            0x01100025 => Ok(Mono14),
            0x010C0026 => Ok(BayerGR10Packed),
            0x010C0027 => Ok(BayerRG10Packed),
            0x010C0028 => Ok(BayerGB10Packed),
            0x010C0029 => Ok(BayerBG10Packed),
            0x010C002A => Ok(BayerGR12Packed),
            0x010C002B => Ok(BayerRG12Packed),
            0x010C002C => Ok(BayerGB12Packed),
            0x010C002D => Ok(BayerBG12Packed),
            0x0110002E => Ok(BayerGR16),
            0x0110002F => Ok(BayerRG16),
            0x01100030 => Ok(BayerGB16),
            0x01100031 => Ok(BayerBG16),
            0x02100032 => Ok(YUV422_8),
            0x02300033 => Ok(RGB16),
            0x02240034 => Ok(RGB12V1Packed),
            0x02100035 => Ok(RGB565p),
            0x02100036 => Ok(BGR565p),
            0x01010037 => Ok(Mono1p),
            0x01020038 => Ok(Mono2p),
            0x01040039 => Ok(Mono4p),
            0x0218003A => Ok(YCbCr8_CbYCr),
            0x0210003B => Ok(YCbCr422_8),
            0x020C003C => Ok(YCbCr411_8_CbYYCrYY),
            0x0218003D => Ok(YCbCr601_8_CbYCr),
            0x0210003E => Ok(YCbCr601_422_8),
            0x020C003F => Ok(YCbCr601_411_8_CbYYCrYY),
            0x02180040 => Ok(YCbCr709_8_CbYCr),
            0x02100041 => Ok(YCbCr709_422_8),
            0x020C0042 => Ok(YCbCr709_411_8_CbYYCrYY),
            0x02100043 => Ok(YCbCr422_8_CbYCrY),
            0x02100044 => Ok(YCbCr601_422_8_CbYCrY),
            0x02100045 => Ok(YCbCr709_422_8_CbYCrY),
            0x010A0046 => Ok(Mono10p),
            0x010C0047 => Ok(Mono12p),
            0x021E0048 => Ok(BGR10p),
            0x02240049 => Ok(BGR12p),
            0x0230004A => Ok(BGR14),
            0x0230004B => Ok(BGR16),
            0x0240004C => Ok(BGRa10),
            0x0228004D => Ok(BGRa10p),
            0x0240004E => Ok(BGRa12),
            0x0230004F => Ok(BGRa12p),
            0x02400050 => Ok(BGRa14),
            0x02400051 => Ok(BGRa16),
            0x010A0052 => Ok(BayerBG10p),
            0x010C0053 => Ok(BayerBG12p),
            0x010A0054 => Ok(BayerGB10p),
            0x010C0055 => Ok(BayerGB12p),
            0x010A0056 => Ok(BayerGR10p),
            0x010C0057 => Ok(BayerGR12p),
            0x010A0058 => Ok(BayerRG10p),
            0x010C0059 => Ok(BayerRG12p),
            0x020C005A => Ok(YCbCr411_8),
            0x0218005B => Ok(YCbCr8),
            0x021E005C => Ok(RGB10p),
            0x0224005D => Ok(RGB12p),
            0x0230005E => Ok(RGB14),
            0x0240005F => Ok(RGBa10),
            0x02280060 => Ok(RGBa10p),
            0x02400061 => Ok(RGBa12),
            0x02300062 => Ok(RGBa12p),
            0x02400063 => Ok(RGBa14),
            0x02400064 => Ok(RGBa16),
            0x02200065 => Ok(YCbCr422_10),
            0x02200066 => Ok(YCbCr422_12),
            0x01080067 => Ok(SCF1WBWG8),
            0x01100068 => Ok(SCF1WBWG10),
            0x010A0069 => Ok(SCF1WBWG10p),
            0x0110006A => Ok(SCF1WBWG12),
            0x010C006B => Ok(SCF1WBWG12p),
            0x0110006C => Ok(SCF1WBWG14),
            0x0110006D => Ok(SCF1WBWG16),
            0x0108006E => Ok(SCF1WGWB8),
            0x0110006F => Ok(SCF1WGWB10),
            0x010A0070 => Ok(SCF1WGWB10p),
            0x01100071 => Ok(SCF1WGWB12),
            0x010C0072 => Ok(SCF1WGWB12p),
            0x01100073 => Ok(SCF1WGWB14),
            0x01100074 => Ok(SCF1WGWB16),
            0x01080075 => Ok(SCF1WGWR8),
            0x01100076 => Ok(SCF1WGWR10),
            0x010A0077 => Ok(SCF1WGWR10p),
            0x01100078 => Ok(SCF1WGWR12),
            0x010C0079 => Ok(SCF1WGWR12p),
            0x0110007A => Ok(SCF1WGWR14),
            0x0110007B => Ok(SCF1WGWR16),
            0x0108007C => Ok(SCF1WRWG8),
            0x0110007D => Ok(SCF1WRWG10),
            0x010A007E => Ok(SCF1WRWG10p),
            0x0110007F => Ok(SCF1WRWG12),
            0x010C0080 => Ok(SCF1WRWG12p),
            0x01100081 => Ok(SCF1WRWG14),
            0x01100082 => Ok(SCF1WRWG16),
            0x02300083 => Ok(YCbCr10_CbYCr),
            0x021E0084 => Ok(YCbCr10p_CbYCr),
            0x02300085 => Ok(YCbCr12_CbYCr),
            0x02240086 => Ok(YCbCr12p_CbYCr),
            0x02140087 => Ok(YCbCr422_10p),
            0x02180088 => Ok(YCbCr422_12p),
            0x02300089 => Ok(YCbCr601_10_CbYCr),
            0x021E008A => Ok(YCbCr601_10p_CbYCr),
            0x0230008B => Ok(YCbCr601_12_CbYCr),
            0x0224008C => Ok(YCbCr601_12p_CbYCr),
            0x0220008D => Ok(YCbCr601_422_10),
            0x0214008E => Ok(YCbCr601_422_10p),
            0x0220008F => Ok(YCbCr601_422_12),
            0x02180090 => Ok(YCbCr601_422_12p),
            0x02300091 => Ok(YCbCr709_10_CbYCr),
            0x021E0092 => Ok(YCbCr709_10p_CbYCr),
            0x02300093 => Ok(YCbCr709_12_CbYCr),
            0x02240094 => Ok(YCbCr709_12p_CbYCr),
            0x02200095 => Ok(YCbCr709_422_10),
            0x02140096 => Ok(YCbCr709_422_10p),
            0x02200097 => Ok(YCbCr709_422_12),
            0x02180098 => Ok(YCbCr709_422_12p),
            0x02200099 => Ok(YCbCr422_10_CbYCrY),
            0x0214009A => Ok(YCbCr422_10p_CbYCrY),
            0x0220009B => Ok(YCbCr422_12_CbYCrY),
            0x0218009C => Ok(YCbCr422_12p_CbYCrY),
            0x0220009D => Ok(YCbCr601_422_10_CbYCrY),
            0x0214009E => Ok(YCbCr601_422_10p_CbYCrY),
            0x0220009F => Ok(YCbCr601_422_12_CbYCrY),
            0x021800A0 => Ok(YCbCr601_422_12p_CbYCrY),
            0x022000A1 => Ok(YCbCr709_422_10_CbYCrY),
            0x021400A2 => Ok(YCbCr709_422_10p_CbYCrY),
            0x022000A3 => Ok(YCbCr709_422_12_CbYCrY),
            0x021800A4 => Ok(YCbCr709_422_12p_CbYCrY),
            0x021000A5 => Ok(BiColorRGBG8),
            0x021000A6 => Ok(BiColorBGRG8),
            0x022000A7 => Ok(BiColorRGBG10),
            0x021400A8 => Ok(BiColorRGBG10p),
            0x022000A9 => Ok(BiColorBGRG10),
            0x021400AA => Ok(BiColorBGRG10p),
            0x022000AB => Ok(BiColorRGBG12),
            0x021800AC => Ok(BiColorRGBG12p),
            0x022000AD => Ok(BiColorBGRG12),
            0x021800AE => Ok(BiColorBGRG12p),
            0x010800AF => Ok(Coord3D_A8),
            0x010800B0 => Ok(Coord3D_B8),
            0x010800B1 => Ok(Coord3D_C8),
            0x021800B2 => Ok(Coord3D_ABC8),
            0x021800B3 => Ok(Coord3D_ABC8_Planar),
            0x021000B4 => Ok(Coord3D_AC8),
            0x021000B5 => Ok(Coord3D_AC8_Planar),
            0x011000B6 => Ok(Coord3D_A16),
            0x011000B7 => Ok(Coord3D_B16),
            0x011000B8 => Ok(Coord3D_C16),
            0x023000B9 => Ok(Coord3D_ABC16),
            0x023000BA => Ok(Coord3D_ABC16_Planar),
            0x022000BB => Ok(Coord3D_AC16),
            0x022000BC => Ok(Coord3D_AC16_Planar),
            0x012000BD => Ok(Coord3D_A32f),
            0x012000BE => Ok(Coord3D_B32f),
            0x012000BF => Ok(Coord3D_C32f),
            0x026000C0 => Ok(Coord3D_ABC32f),
            0x026000C1 => Ok(Coord3D_ABC32f_Planar),
            0x024000C2 => Ok(Coord3D_AC32f),
            0x024000C3 => Ok(Coord3D_AC32f_Planar),
            0x010800C4 => Ok(Confidence1),
            0x010100C5 => Ok(Confidence1p),
            0x010800C6 => Ok(Confidence8),
            0x011000C7 => Ok(Confidence16),
            0x012000C8 => Ok(Confidence32f),
            0x010800C9 => Ok(R8),
            0x010A00CA => Ok(R10),
            0x010C00CB => Ok(R12),
            0x011000CC => Ok(R16),
            0x010800CD => Ok(G8),
            0x010A00CE => Ok(G10),
            0x010C00CF => Ok(G12),
            0x011000D0 => Ok(G16),
            0x010800D1 => Ok(B8),
            0x010A00D2 => Ok(B10),
            0x010C00D3 => Ok(B12),
            0x011000D4 => Ok(B16),
            0x010A00D5 => Ok(Coord3D_A10p),
            0x010A00D6 => Ok(Coord3D_B10p),
            0x010A00D7 => Ok(Coord3D_C10p),
            0x010C00D8 => Ok(Coord3D_A12p),
            0x010C00D9 => Ok(Coord3D_B12p),
            0x010C00DA => Ok(Coord3D_C12p),
            0x021E00DB => Ok(Coord3D_ABC10p),
            0x021E00DC => Ok(Coord3D_ABC10p_Planar),
            0x022400DE => Ok(Coord3D_ABC12p),
            0x022400DF => Ok(Coord3D_ABC12p_Planar),
            0x021400F0 => Ok(Coord3D_AC10p),
            0x021400F1 => Ok(Coord3D_AC10p_Planar),
            0x021800F2 => Ok(Coord3D_AC12p),
            0x021800F3 => Ok(Coord3D_AC12p_Planar),
            0x021800F4 => Ok(YCbCr2020_8_CbYCr),
            0x023000F5 => Ok(YCbCr2020_10_CbYCr),
            0x021E00F6 => Ok(YCbCr2020_10p_CbYCr),
            0x023000F7 => Ok(YCbCr2020_12_CbYCr),
            0x022400F8 => Ok(YCbCr2020_12p_CbYCr),
            0x020C00F9 => Ok(YCbCr2020_411_8_CbYYCrYY),
            0x021000FA => Ok(YCbCr2020_422_8),
            0x021000FB => Ok(YCbCr2020_422_8_CbYCrY),
            0x022000FC => Ok(YCbCr2020_422_10),
            0x022000FD => Ok(YCbCr2020_422_10_CbYCrY),
            0x021400FE => Ok(YCbCr2020_422_10p),
            0x021400FF => Ok(YCbCr2020_422_10p_CbYCrY),
            0x02200100 => Ok(YCbCr2020_422_12),
            0x02200101 => Ok(YCbCr2020_422_12_CbYCrY),
            0x02180102 => Ok(YCbCr2020_422_12p),
            0x02180103 => Ok(YCbCr2020_422_12p_CbYCrY),
            0x010E0104 => Ok(Mono14p),
            0x010E0105 => Ok(BayerGR14p),
            0x010E0106 => Ok(BayerRG14p),
            0x010E0107 => Ok(BayerGB14p),
            0x010E0108 => Ok(BayerBG14p),
            0x01100109 => Ok(BayerGR14),
            0x0110010A => Ok(BayerRG14),
            0x0110010B => Ok(BayerGB14),
            0x0110010C => Ok(BayerBG14),
            0x0104010D => Ok(BayerGR4p),
            0x0104010E => Ok(BayerRG4p),
            0x0104010F => Ok(BayerGB4p),
            0x01040110 => Ok(BayerBG4p),
            0x01200111 => Ok(Mono32),
            0x020C0112 => Ok(YCbCr420_8_YY_CbCr_Semiplanar),
            0x02100113 => Ok(YCbCr422_8_YY_CbCr_Semiplanar),
            0x020C0114 => Ok(YCbCr420_8_YY_CrCb_Semiplanar),
            0x02100115 => Ok(YCbCr422_8_YY_CrCb_Semiplanar),
            0x01080116 => Ok(Data8),
            0x01080117 => Ok(Data8s),
            0x01100118 => Ok(Data16),
            0x01100119 => Ok(Data16s),
            0x0120011A => Ok(Data32),
            0x0120011B => Ok(Data32s),
            0x0120011C => Ok(Data32f),
            0x0140011D => Ok(Data64),
            0x0140011E => Ok(Data64s),
            0x0140011F => Ok(Data64f),
            otherwise => Err(format!("{:x} is invalid value for pixel format", otherwise)),
        }
    }
}

impl Into<u32> for PixelFormat {
    fn into(self) -> u32 {
        use PixelFormat::*;
        match self {
            Mono8 => 0x01080001,
            Mono8s => 0x01080002,
            Mono10 => 0x01100003,
            Mono10Packed => 0x010C0004,
            Mono12 => 0x01100005,
            Mono12Packed => 0x010C0006,
            Mono16 => 0x01100007,
            BayerGR8 => 0x01080008,
            BayerRG8 => 0x01080009,
            BayerGB8 => 0x0108000A,
            BayerBG8 => 0x0108000B,
            BayerGR10 => 0x0110000C,
            BayerRG10 => 0x0110000D,
            BayerGB10 => 0x0110000E,
            BayerBG10 => 0x0110000F,
            BayerGR12 => 0x01100010,
            BayerRG12 => 0x01100011,
            BayerGB12 => 0x01100012,
            BayerBG12 => 0x01100013,
            RGB8 => 0x02180014,
            BGR8 => 0x02180015,
            RGBa8 => 0x02200016,
            BGRa8 => 0x02200017,
            RGB10 => 0x02300018,
            BGR10 => 0x02300019,
            RGB12 => 0x0230001A,
            BGR12 => 0x0230001B,
            YUV8_UYV => 0x02180020,
            RGB8_Planar => 0x02180021,
            RGB10_Planar => 0x02300022,
            RGB12_Planar => 0x02300023,
            RGB16_Planar => 0x02300024,
            Mono14 => 0x01100025,
            BayerGR10Packed => 0x010C0026,
            BayerRG10Packed => 0x010C0027,
            BayerGB10Packed => 0x010C0028,
            BayerBG10Packed => 0x010C0029,
            BayerGR12Packed => 0x010C002A,
            BayerRG12Packed => 0x010C002B,
            BayerGB12Packed => 0x010C002C,
            BayerBG12Packed => 0x010C002D,
            BayerGR16 => 0x0110002E,
            BayerRG16 => 0x0110002F,
            BayerGB16 => 0x01100030,
            BayerBG16 => 0x01100031,
            YUV422_8 => 0x02100032,
            RGB16 => 0x02300033,
            RGB12V1Packed => 0x02240034,
            RGB565p => 0x02100035,
            BGR565p => 0x02100036,
            Mono1p => 0x01010037,
            Mono2p => 0x01020038,
            Mono4p => 0x01040039,
            YCbCr8_CbYCr => 0x0218003A,
            YCbCr422_8 => 0x0210003B,
            YCbCr411_8_CbYYCrYY => 0x020C003C,
            YCbCr601_8_CbYCr => 0x0218003D,
            YCbCr601_422_8 => 0x0210003E,
            YCbCr601_411_8_CbYYCrYY => 0x020C003F,
            YCbCr709_8_CbYCr => 0x02180040,
            YCbCr709_422_8 => 0x02100041,
            YCbCr709_411_8_CbYYCrYY => 0x020C0042,
            YCbCr422_8_CbYCrY => 0x02100043,
            YCbCr601_422_8_CbYCrY => 0x02100044,
            YCbCr709_422_8_CbYCrY => 0x02100045,
            Mono10p => 0x010A0046,
            Mono12p => 0x010C0047,
            BGR10p => 0x021E0048,
            BGR12p => 0x02240049,
            BGR14 => 0x0230004A,
            BGR16 => 0x0230004B,
            BGRa10 => 0x0240004C,
            BGRa10p => 0x0228004D,
            BGRa12 => 0x0240004E,
            BGRa12p => 0x0230004F,
            BGRa14 => 0x02400050,
            BGRa16 => 0x02400051,
            BayerBG10p => 0x010A0052,
            BayerBG12p => 0x010C0053,
            BayerGB10p => 0x010A0054,
            BayerGB12p => 0x010C0055,
            BayerGR10p => 0x010A0056,
            BayerGR12p => 0x010C0057,
            BayerRG10p => 0x010A0058,
            BayerRG12p => 0x010C0059,
            YCbCr411_8 => 0x020C005A,
            YCbCr8 => 0x0218005B,
            RGB10p => 0x021E005C,
            RGB12p => 0x0224005D,
            RGB14 => 0x0230005E,
            RGBa10 => 0x0240005F,
            RGBa10p => 0x02280060,
            RGBa12 => 0x02400061,
            RGBa12p => 0x02300062,
            RGBa14 => 0x02400063,
            RGBa16 => 0x02400064,
            YCbCr422_10 => 0x02200065,
            YCbCr422_12 => 0x02200066,
            SCF1WBWG8 => 0x01080067,
            SCF1WBWG10 => 0x01100068,
            SCF1WBWG10p => 0x010A0069,
            SCF1WBWG12 => 0x0110006A,
            SCF1WBWG12p => 0x010C006B,
            SCF1WBWG14 => 0x0110006C,
            SCF1WBWG16 => 0x0110006D,
            SCF1WGWB8 => 0x0108006E,
            SCF1WGWB10 => 0x0110006F,
            SCF1WGWB10p => 0x010A0070,
            SCF1WGWB12 => 0x01100071,
            SCF1WGWB12p => 0x010C0072,
            SCF1WGWB14 => 0x01100073,
            SCF1WGWB16 => 0x01100074,
            SCF1WGWR8 => 0x01080075,
            SCF1WGWR10 => 0x01100076,
            SCF1WGWR10p => 0x010A0077,
            SCF1WGWR12 => 0x01100078,
            SCF1WGWR12p => 0x010C0079,
            SCF1WGWR14 => 0x0110007A,
            SCF1WGWR16 => 0x0110007B,
            SCF1WRWG8 => 0x0108007C,
            SCF1WRWG10 => 0x0110007D,
            SCF1WRWG10p => 0x010A007E,
            SCF1WRWG12 => 0x0110007F,
            SCF1WRWG12p => 0x010C0080,
            SCF1WRWG14 => 0x01100081,
            SCF1WRWG16 => 0x01100082,
            YCbCr10_CbYCr => 0x02300083,
            YCbCr10p_CbYCr => 0x021E0084,
            YCbCr12_CbYCr => 0x02300085,
            YCbCr12p_CbYCr => 0x02240086,
            YCbCr422_10p => 0x02140087,
            YCbCr422_12p => 0x02180088,
            YCbCr601_10_CbYCr => 0x02300089,
            YCbCr601_10p_CbYCr => 0x021E008A,
            YCbCr601_12_CbYCr => 0x0230008B,
            YCbCr601_12p_CbYCr => 0x0224008C,
            YCbCr601_422_10 => 0x0220008D,
            YCbCr601_422_10p => 0x0214008E,
            YCbCr601_422_12 => 0x0220008F,
            YCbCr601_422_12p => 0x02180090,
            YCbCr709_10_CbYCr => 0x02300091,
            YCbCr709_10p_CbYCr => 0x021E0092,
            YCbCr709_12_CbYCr => 0x02300093,
            YCbCr709_12p_CbYCr => 0x02240094,
            YCbCr709_422_10 => 0x02200095,
            YCbCr709_422_10p => 0x02140096,
            YCbCr709_422_12 => 0x02200097,
            YCbCr709_422_12p => 0x02180098,
            YCbCr422_10_CbYCrY => 0x02200099,
            YCbCr422_10p_CbYCrY => 0x0214009A,
            YCbCr422_12_CbYCrY => 0x0220009B,
            YCbCr422_12p_CbYCrY => 0x0218009C,
            YCbCr601_422_10_CbYCrY => 0x0220009D,
            YCbCr601_422_10p_CbYCrY => 0x0214009E,
            YCbCr601_422_12_CbYCrY => 0x0220009F,
            YCbCr601_422_12p_CbYCrY => 0x021800A0,
            YCbCr709_422_10_CbYCrY => 0x022000A1,
            YCbCr709_422_10p_CbYCrY => 0x021400A2,
            YCbCr709_422_12_CbYCrY => 0x022000A3,
            YCbCr709_422_12p_CbYCrY => 0x021800A4,
            BiColorRGBG8 => 0x021000A5,
            BiColorBGRG8 => 0x021000A6,
            BiColorRGBG10 => 0x022000A7,
            BiColorRGBG10p => 0x021400A8,
            BiColorBGRG10 => 0x022000A9,
            BiColorBGRG10p => 0x021400AA,
            BiColorRGBG12 => 0x022000AB,
            BiColorRGBG12p => 0x021800AC,
            BiColorBGRG12 => 0x022000AD,
            BiColorBGRG12p => 0x021800AE,
            Coord3D_A8 => 0x010800AF,
            Coord3D_B8 => 0x010800B0,
            Coord3D_C8 => 0x010800B1,
            Coord3D_ABC8 => 0x021800B2,
            Coord3D_ABC8_Planar => 0x021800B3,
            Coord3D_AC8 => 0x021000B4,
            Coord3D_AC8_Planar => 0x021000B5,
            Coord3D_A16 => 0x011000B6,
            Coord3D_B16 => 0x011000B7,
            Coord3D_C16 => 0x011000B8,
            Coord3D_ABC16 => 0x023000B9,
            Coord3D_ABC16_Planar => 0x023000BA,
            Coord3D_AC16 => 0x022000BB,
            Coord3D_AC16_Planar => 0x022000BC,
            Coord3D_A32f => 0x012000BD,
            Coord3D_B32f => 0x012000BE,
            Coord3D_C32f => 0x012000BF,
            Coord3D_ABC32f => 0x026000C0,
            Coord3D_ABC32f_Planar => 0x026000C1,
            Coord3D_AC32f => 0x024000C2,
            Coord3D_AC32f_Planar => 0x024000C3,
            Confidence1 => 0x010800C4,
            Confidence1p => 0x010100C5,
            Confidence8 => 0x010800C6,
            Confidence16 => 0x011000C7,
            Confidence32f => 0x012000C8,
            R8 => 0x010800C9,
            R10 => 0x010A00CA,
            R12 => 0x010C00CB,
            R16 => 0x011000CC,
            G8 => 0x010800CD,
            G10 => 0x010A00CE,
            G12 => 0x010C00CF,
            G16 => 0x011000D0,
            B8 => 0x010800D1,
            B10 => 0x010A00D2,
            B12 => 0x010C00D3,
            B16 => 0x011000D4,
            Coord3D_A10p => 0x010A00D5,
            Coord3D_B10p => 0x010A00D6,
            Coord3D_C10p => 0x010A00D7,
            Coord3D_A12p => 0x010C00D8,
            Coord3D_B12p => 0x010C00D9,
            Coord3D_C12p => 0x010C00DA,
            Coord3D_ABC10p => 0x021E00DB,
            Coord3D_ABC10p_Planar => 0x021E00DC,
            Coord3D_ABC12p => 0x022400DE,
            Coord3D_ABC12p_Planar => 0x022400DF,
            Coord3D_AC10p => 0x021400F0,
            Coord3D_AC10p_Planar => 0x021400F1,
            Coord3D_AC12p => 0x021800F2,
            Coord3D_AC12p_Planar => 0x021800F3,
            YCbCr2020_8_CbYCr => 0x021800F4,
            YCbCr2020_10_CbYCr => 0x023000F5,
            YCbCr2020_10p_CbYCr => 0x021E00F6,
            YCbCr2020_12_CbYCr => 0x023000F7,
            YCbCr2020_12p_CbYCr => 0x022400F8,
            YCbCr2020_411_8_CbYYCrYY => 0x020C00F9,
            YCbCr2020_422_8 => 0x021000FA,
            YCbCr2020_422_8_CbYCrY => 0x021000FB,
            YCbCr2020_422_10 => 0x022000FC,
            YCbCr2020_422_10_CbYCrY => 0x022000FD,
            YCbCr2020_422_10p => 0x021400FE,
            YCbCr2020_422_10p_CbYCrY => 0x021400FF,
            YCbCr2020_422_12 => 0x02200100,
            YCbCr2020_422_12_CbYCrY => 0x02200101,
            YCbCr2020_422_12p => 0x02180102,
            YCbCr2020_422_12p_CbYCrY => 0x02180103,
            Mono14p => 0x010E0104,
            BayerGR14p => 0x010E0105,
            BayerRG14p => 0x010E0106,
            BayerGB14p => 0x010E0107,
            BayerBG14p => 0x010E0108,
            BayerGR14 => 0x01100109,
            BayerRG14 => 0x0110010A,
            BayerGB14 => 0x0110010B,
            BayerBG14 => 0x0110010C,
            BayerGR4p => 0x0104010D,
            BayerRG4p => 0x0104010E,
            BayerGB4p => 0x0104010F,
            BayerBG4p => 0x01040110,
            Mono32 => 0x01200111,
            YCbCr420_8_YY_CbCr_Semiplanar => 0x020C0112,
            YCbCr422_8_YY_CbCr_Semiplanar => 0x02100113,
            YCbCr420_8_YY_CrCb_Semiplanar => 0x020C0114,
            YCbCr422_8_YY_CrCb_Semiplanar => 0x02100115,
            Data8 => 0x01080116,
            Data8s => 0x01080117,
            Data16 => 0x01100118,
            Data16s => 0x01100119,
            Data32 => 0x0120011A,
            Data32s => 0x0120011B,
            Data32f => 0x0120011C,
            Data64 => 0x0140011D,
            Data64s => 0x0140011E,
            Data64f => 0x0140011F,
        }
    }
}
