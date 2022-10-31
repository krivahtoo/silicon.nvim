use std::path::PathBuf;

use nvim_oxi as oxi;
use oxi::{FromObject, FromObjectResult, Object, ToObject, ToObjectResult};
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

    pub output: Option<PathBuf>,

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
    fn from_obj(obj: Object) -> FromObjectResult<Self> {
        Self::deserialize(oxi::Deserializer::new(obj)).map_err(Into::into)
    }
}

impl ToObject for Opts {
    fn to_obj(self) -> ToObjectResult {
        self.serialize(oxi::Serializer::new()).map_err(Into::into)
    }
}

impl oxi::lua::Poppable for Opts {
    unsafe fn pop(lstate: *mut oxi::lua::ffi::lua_State) -> Result<Self, oxi::lua::Error> {
        let obj = Object::pop(lstate)?;
        Self::from_obj(obj).map_err(oxi::lua::Error::pop_error_from_err::<Self, _>)
    }
}

impl oxi::lua::Pushable for Opts {
    unsafe fn push(
        self,
        lstate: *mut oxi::lua::ffi::lua_State,
    ) -> Result<std::ffi::c_int, oxi::lua::Error> {
        self.to_obj()
            .map_err(oxi::lua::Error::push_error_from_err::<Self, _>)?
            .push(lstate)
    }
}

