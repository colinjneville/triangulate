
const ROOT_PREFIX: &str = "TRIANGULATE";

pub(crate) mod svg {
    use std::{env, path};

    use crate::debug;

    const GROUP_PREFIX: &str = "SVG";
    
    pub(crate) fn output_path() -> Option<path::PathBuf> {
        const KEY: &str = "OUTPUT_PATH";
        let key = format!("{}_{}_{}", super::ROOT_PREFIX, GROUP_PREFIX, KEY);
        
        if let Ok(value) = env::var(key) {
            Some(path::PathBuf::from(value))
        } else {
            None
        }
    }
    
    pub(crate) fn show_labels() -> bool {
        // Note the show/hide inversion
        const KEY: &str = "HIDE_LABELS";
        let key = format!("{}_{}_{}", super::ROOT_PREFIX, GROUP_PREFIX, KEY);

        !env::var(key).is_ok()
    }

    pub(crate) fn output_level() -> debug::svg::SvgOutputLevel {
        const KEY: &str = "OUTPUT_LEVEL";
        let key = format!("{}_{}_{}", super::ROOT_PREFIX, GROUP_PREFIX, KEY);

        use debug::svg::SvgOutputLevel;

        match env::var(key) {
            Ok(value) => {
                match value.to_ascii_lowercase().as_str() {
                    "3" => SvgOutputLevel::AllSteps,
                    "2" => SvgOutputLevel::MajorSteps,
                    "1" => SvgOutputLevel::ResultOnly,
                    "0" | _ => SvgOutputLevel::None,
                }
            }
            Err(_) => SvgOutputLevel::None,
        }
    }
}
