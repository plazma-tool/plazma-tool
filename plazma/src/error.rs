use std::{fmt, error};
use std::str;
use std::error::Error;
use intro_runtime::error::RuntimeError;

pub enum ToolError {
    Runtime(RuntimeError, String),
    AudioTrackDoesntExists(String),
    NameAlreadyExists,
    NoQuad,
    UiError(String),
    MissingProjectRoot,
    MissingObjectPath,
}

pub enum ToolOk {
    ShaderCompilationSuccess,
    AssetCompilationSuccess,
}

impl fmt::Display for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl fmt::Display for ToolOk {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl fmt::Debug for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        use intro_runtime::error::RuntimeError::*;

        let kind: &'static str = match *self {
            ToolError::Runtime(ref e, _) => match *e {
                ShaderCompilationFailed => "Shader compilation failed",
                ShaderLinkingFailed => "Shader linking failed",
                ShaderInfoLogIsNotValidUTF8 => "Shader info log is not valid UTF8",
                SceneIdxIsOutOfBounds => "scene_idx is out of bounds",
                FailedToCreateNoSuchFragSrcIdx => "Failed to create: No such frag_src idx",
                FailedToCreateNoSuchVertSrcIdx => "Failed to create: No such vert_src idx",
                FrameBufferIsNotComplete => "Frame buffer is not complete",
                FrameBufferPixelDataIsMissing => "Frame buffer pixel data is missing",
                FrameBufferPixelFormatIsMissing => "Frame buffer pixel format is missing",
                ImageIndexIsOutOfBounds => "Image idx is out of bounds",
                ContextIndexIsOutOfBounds => "Content idx is out of bounds",
                ShaderSourceIdxIsOutOfBounds => "Shader source idx is out of bounds",
                NoId => "No Id",
                NoFbo => "No FBO",
                NoUbo => "No UBO",
                NoQuad => "No Quad",
                TextureBindingIdxIsOverTheHardwareLimit =>
                    "Texture binding idx is over the hardware limit",
                TexturePixelDataIsMissing =>
                    "Texture pixel data is missing",
                UniformBlockBindingIdxIsOverTheHardwareLimit =>
                    "Uniform block binding idx is over the hardware limit",
                TextureBindingIdxDoesntExist => "Texture binding idx doesn't exist",
                TrackIdxIsOutOfBounds => "Track idx is out of bounds",
                VarIdxIsOutOfBounds => "Var idx is out of bounds",
                DataIdxIsOutOfBounds => "Data idx is out of bounds",
                CantOpenImage => "Can't open image",
            },

            ToolError::AudioTrackDoesntExists(_) => "Audio track doesn't exists",
            ToolError::NoQuad => "No Quad",
            ToolError::UiError(_) => "UI Error",
            ToolError::NameAlreadyExists => "Name already exists",
            ToolError::MissingProjectRoot => "Missing project root",
            ToolError::MissingObjectPath => "Missing object path",
        };

        write!(fmt, "{}:\n{}", kind, self.description())
    }
}

impl fmt::Debug for ToolOk {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let kind: &'static str = match *self {
            ToolOk::ShaderCompilationSuccess => "Shader compilation success",
            ToolOk::AssetCompilationSuccess => "Asset compilation success",
        };

        write!(fmt, "{}", kind)
    }
}

impl error::Error for ToolError {
    fn description(&self) -> &str {
        match *self {
            ToolError::Runtime(_, ref s) => s.trim(),
            ToolError::AudioTrackDoesntExists(ref s) =>  s.trim(),
            ToolError::NoQuad => "No Quad",
            ToolError::UiError(ref s) => s.trim(),
            ToolError::NameAlreadyExists => "Name already exists",
            ToolError::MissingProjectRoot => "Missing project root",
            ToolError::MissingObjectPath => "Missing object path",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
