use std::{ffi, path::PathBuf};

use nvim_oxi as oxi;
use oxi::{
    conversion::{self, FromObject, ToObject},
    lua,
    serde::{Deserializer, Serializer},
    Object,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct WatermarkOpts {
    pub text: Option<String>,
    pub font: Option<String>,
    pub color: Option<String>,
    pub style: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ShadowOpts {
    #[serde(default)]
    pub blur_radius: f32,
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
    pub color: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct OutputOpts {
    pub file: Option<PathBuf>,
    pub clipboard: Option<bool>,
    pub path: Option<PathBuf>,
    pub format: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Opts {
    pub font: Option<String>,

    pub theme: Option<String>,

    pub background: Option<String>,

    #[serde(default)]
    pub shadow: ShadowOpts,

    pub pad_horiz: Option<u32>,
    pub pad_vert: Option<u32>,

    pub line_number: Option<bool>,
    pub line_pad: Option<u32>,
    pub line_offset: Option<u32>,

    pub tab_width: Option<u8>,

    pub round_corner: Option<bool>,
    pub window_controls: Option<bool>,

    #[serde(default)]
    pub output: OutputOpts,

    #[serde(default)]
    pub watermark: WatermarkOpts,

    #[serde(alias = "line1")]
    #[serde(default)]
    pub start: usize,
    #[serde(alias = "line2")]
    #[serde(default)]
    pub end: usize,
}

impl FromObject for Opts {
    fn from_object(obj: Object) -> Result<Self, conversion::Error> {
        Self::deserialize(Deserializer::new(obj)).map_err(Into::into)
    }
}

impl ToObject for Opts {
    fn to_object(self) -> Result<Object, conversion::Error> {
        self.serialize(Serializer::new()).map_err(Into::into)
    }
}

impl lua::Poppable for Opts {
    unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
        let obj = Object::pop(lstate)?;
        Self::from_object(obj).map_err(lua::Error::pop_error_from_err::<Self, _>)
    }
}

impl lua::Pushable for Opts {
    unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<ffi::c_int, lua::Error> {
        self.to_object()
            .map_err(lua::Error::push_error_from_err::<Self, _>)?
            .push(lstate)
    }
}
