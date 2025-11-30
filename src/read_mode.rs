// src/read_mode.rs
use serde::{Deserialize, Serialize};

/// Common enum for describing read mode in YAML.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum YamlReadMode {
    #[serde(rename = "lines")]
    Lines {
        max_line_len: Option<usize>,
    },
    #[serde(rename = "fixed_size")]
    FixedSize {
        frame_size: usize,
    },
    #[serde(rename = "delimited")]
    Delimited {
        /// Delimiter byte (0-255), e.g. 10 == '\n'
        delim: u8,
        max_len: Option<usize>,
    },
    #[serde(rename = "length_prefixed")]
    LengthPrefixed {
        /// Usually 1, 2, or 4 bytes for length
        len_bytes: usize,
        big_endian: bool,
        max_len: Option<usize>,
    },
}

/// Context that goes directly into the template (handlebars).
/// It's convenient to `flatten` it into TemplateCtx.
#[derive(Debug, Default, Serialize)]
pub struct ReadModeTemplateCtx {
    // lines mode
    pub max_line_len: Option<usize>,

    // fixed_size mode
    pub frame_size: Option<usize>,

    // delimited mode
    pub delim_byte: Option<u8>,
    pub delim_max_len: Option<usize>,

    // length_prefixed mode
    pub lp_len_bytes: Option<usize>,
    pub lp_big_endian: Option<bool>,
    pub lp_max_len: Option<usize>,
    pub lp_parse_len_code: String,

    // mode flags
    pub is_lines: bool,
    pub is_fixed_size: bool,
    pub is_delimited: bool,
    pub is_length_prefixed: bool,
}

impl From<YamlReadMode> for ReadModeTemplateCtx {
    fn from(mode: YamlReadMode) -> Self {
        let mut ctx = ReadModeTemplateCtx::default();

        match mode {
            YamlReadMode::Lines { max_line_len } => {
                ctx.is_lines = true;
                ctx.max_line_len = max_line_len;
            }
            YamlReadMode::FixedSize { frame_size } => {
                ctx.is_fixed_size = true;
                ctx.frame_size = Some(frame_size);
            }
            YamlReadMode::Delimited { delim, max_len } => {
                ctx.is_delimited = true;
                ctx.delim_byte = Some(delim);
                ctx.delim_max_len = max_len;
            }
            YamlReadMode::LengthPrefixed {
                len_bytes,
                big_endian,
                max_len,
            } => {
                ctx.is_length_prefixed = true;
                ctx.lp_len_bytes = Some(len_bytes);
                ctx.lp_big_endian = Some(big_endian);
                ctx.lp_max_len = max_len;
                ctx.lp_parse_len_code = lp_parse_len_snippet(len_bytes, big_endian);
            }
        }

        ctx
    }
}

fn lp_parse_len_snippet(len_bytes: usize, big_endian: bool) -> String {
    match (len_bytes, big_endian) {
        (1, _) => "let frame_len: usize = len_buf[0] as usize;".to_string(),
        (2, true) =>
            "let frame_len: usize = u16::from_be_bytes([len_buf[0], len_buf[1]]) as usize;".to_string(),
        (2, false) =>
            "let frame_len: usize = u16::from_le_bytes([len_buf[0], len_buf[1]]) as usize;".to_string(),
        (4, true) =>
            "let frame_len: usize = u32::from_be_bytes([len_buf[0], len_buf[1], len_buf[2], len_buf[3]]) as usize;".to_string(),
        (4, false) =>
            "let frame_len: usize = u32::from_le_bytes([len_buf[0], len_buf[1], len_buf[2], len_buf[3]]) as usize;".to_string(),
        _ => panic!("len_bytes must be 1, 2 or 4"),
    }
}
