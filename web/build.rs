use std::{env, io};
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

use chocodye::{Dye, Lang, message};

pub fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("missing `OUT_DIR`"));
    
    for lang in Lang::VALUES {
        let path = out_dir.join(format!("LANG_{}.html", lang.short_code().to_ascii_uppercase()));
        let mut f = File::create(path)?;
        
        let bundle = lang.into_bundle();
        
        let mut dyes = Dye::VALUES;
        dyes.sort_unstable_by_key(|dye| dye.color_name(&bundle));
        
        for (id, label, default) in [("start-select", "starting-color-input", Dye::DEFAULT_CHOCOBO_COLOR), ("final-select", "final-color-input", Dye::InkBlue)] {
            writeln!(f, r#"<label for="{}">{}</label>"#, id, message!(&bundle, label))?;
            write!(f, r#"<select id="{}" onchange="calculate()">"#, id)?;
            
            for dye in dyes.iter().copied() {
                write!(f, r#"<option value="{}" {}>{}</option>"#, dye as u8, if dye == default { "selected" } else { "" }, dye.color_name(&bundle))?;
            }
            
            writeln!(f, r#"</select>"#)?;
        }
    }
    
    Ok(())
}
