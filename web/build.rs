use std::{env, io};
use std::borrow::Cow;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Stdio};
use chrono::{TimeZone, Utc};
use chocodye::{Dye, Lang, message};

pub fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("missing `OUT_DIR`"));

    println!(
        "cargo:rustc-env=GIT_HEAD={}",

        option_env!("GITHUB_SHA").map_or_else(|| {
            let output = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .stderr(Stdio::inherit())
                .output()
                .expect("could not spawn git")
                .stdout;

            Cow::Owned(String::from_utf8(output).expect("git rev-parse returned non-UTF8"))
        }, |sha| Cow::Borrowed(&sha[..7]))
    );

    println!(
        "cargo:rustc-env=BUILD_DATE={}",

        option_env!("SOURCE_DATE_EPOCH").map_or_else(Utc::now, |epoch| {
            Utc.timestamp_opt(epoch.parse().expect("SOURCE_DATE_EPOCH"), 0).unwrap()
        }).format("%Y-%m-%d")
    );

    // HTMLize fluent data
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
            
            writeln!(f, r#"</select><br />"#)?;
        }
    }
    
    Ok(())
}
