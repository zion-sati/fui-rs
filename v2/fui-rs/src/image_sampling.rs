use crate::ffi::ImageSamplingKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSamplingMode {
    Linear,
    Nearest,
    LinearMipmapNearest,
    LinearMipmapLinear,
    CubicMitchell,
    CubicCatmullRom,
    Anisotropic,
}

impl ImageSamplingMode {
    pub(crate) fn ffi_kind(self) -> ImageSamplingKind {
        match self {
            Self::Linear => ImageSamplingKind::Linear,
            Self::Nearest => ImageSamplingKind::Nearest,
            Self::LinearMipmapNearest => ImageSamplingKind::LinearMipmapNearest,
            Self::LinearMipmapLinear => ImageSamplingKind::LinearMipmapLinear,
            Self::CubicMitchell => ImageSamplingKind::CubicMitchell,
            Self::CubicCatmullRom => ImageSamplingKind::CubicCatmullRom,
            Self::Anisotropic => ImageSamplingKind::Anisotropic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSampling {
    kind: ImageSamplingMode,
    max_aniso: u32,
}

impl ImageSampling {
    pub fn linear() -> Self {
        Self::new(ImageSamplingMode::Linear, 0)
    }

    pub fn nearest() -> Self {
        Self::new(ImageSamplingMode::Nearest, 0)
    }

    pub fn linear_mipmap_nearest() -> Self {
        Self::new(ImageSamplingMode::LinearMipmapNearest, 0)
    }

    pub fn linear_mipmap_linear() -> Self {
        Self::new(ImageSamplingMode::LinearMipmapLinear, 0)
    }

    pub fn cubic_mitchell() -> Self {
        Self::new(ImageSamplingMode::CubicMitchell, 0)
    }

    pub fn cubic_catmull_rom() -> Self {
        Self::new(ImageSamplingMode::CubicCatmullRom, 0)
    }

    pub fn anisotropic(max_aniso: u32) -> Self {
        Self::new(ImageSamplingMode::Anisotropic, max_aniso)
    }

    pub fn new(kind: ImageSamplingMode, max_aniso: u32) -> Self {
        Self { kind, max_aniso }
    }

    pub fn kind(self) -> ImageSamplingMode {
        self.kind
    }

    pub(crate) fn ffi_kind(self) -> ImageSamplingKind {
        self.kind.ffi_kind()
    }

    pub fn max_aniso(self) -> u32 {
        self.max_aniso
    }
}
