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
        
        // Generate: lang select
        writeln!(f, r#"<label for="lang-select">{}</label>"#, message!(&bundle, "lang-input"))?;
        writeln!(f,
r#"<select id="lang-select" onchange="updateLang(parseInt(this.value))">
    <option lang="de" value="2">Deutsch</option>
    <option lang="en" value="0" selected>English</option>
    <option lang="fr" value="1">Français</option>
    <option lang="jp" value="3">日本語</option>
</select><br />"#)?;
        
        // Sort: dyes.
        let mut dyes = Dye::VALUES;
        dyes.sort_unstable_by_key(|dye| dye.color_name(&bundle));
        
        // Generate: localized dyes select
        const DYE_SELECTS: [(&str, &str, Dye); 2] = [
            ("start-select", "starting-color-input", Dye::DesertYellow),
            ("final-select", "final-color-input", Dye::InkBlue)
        ];
        
        for (select_id, message_id, default) in DYE_SELECTS {
            writeln!(f, r#"<label for="{}">{}</label>"#, select_id, message!(&bundle, message_id))?;
            write!(f, r#"<select id="{}" onchange="calculate()">"#, select_id)?;
            
            for dye in dyes.iter().copied() {
                write!(f, r#"<option value="{}" {}>{}</option>"#, dye as u8, if dye == default { "selected" } else { "" }, dye.color_name(&bundle))?;
            }
            
            writeln!(f, r#"</select>"#)?;
        }
    }
    
    Ok(())
}
