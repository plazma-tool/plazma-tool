use std::fmt;

pub enum RuntimeError {
    ShaderCompilationFailed,
    ShaderLinkingFailed,
    ShaderInfoLogIsNotValidUTF8,
    SceneIdxIsOutOfBounds,
    FailedToCreateNoSuchFragSrcIdx,
    FailedToCreateNoSuchVertSrcIdx,
    FrameBufferIsNotComplete,
    FrameBufferPixelDataIsMissing,
    FrameBufferPixelFormatIsMissing,
    ImageIndexIsOutOfBounds,
    ContextIndexIsOutOfBounds,
    ShaderSourceIdxIsOutOfBounds,
    NoId,
    NoFbo,
    NoUbo,
    NoQuad,
    TextureBindingIdxIsOverTheHardwareLimit,
    UniformBlockBindingIdxIsOverTheHardwareLimit,
    TextureBindingIdxDoesntExist,
    TexturePixelDataIsMissing,
    TrackIdxIsOutOfBounds,
    VarIdxIsOutOfBounds,
    DataIdxIsOutOfBounds,
    CantOpenImage,
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "That's a RuntimeError")
    }
}
