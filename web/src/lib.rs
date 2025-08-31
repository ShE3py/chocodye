use std::fmt::Write as _;

use chocodye::{Dye, Lang, message, SnackList};

static LANG_DE: &str = include_str!(concat!(env!("OUT_DIR"), "/LANG_DE.html"));
static LANG_EN: &str = include_str!(concat!(env!("OUT_DIR"), "/LANG_EN.html"));
static LANG_FR: &str = include_str!(concat!(env!("OUT_DIR"), "/LANG_FR.html"));
static LANG_JP: &str = include_str!(concat!(env!("OUT_DIR"), "/LANG_JP.html"));

#[no_mangle]
pub static LANGS: [&str; 4] = [LANG_EN, LANG_FR, LANG_DE, LANG_JP];

#[no_mangle]
pub static LANG_SIZES: [usize; 4] = [LANG_EN.len(), LANG_FR.len(), LANG_DE.len(), LANG_JP.len()];

#[no_mangle]
pub extern "C" fn make_meal(starting_dye: i32, final_dye: i32) -> Option<SnackList> {
    let starting_dye = Dye::VALUES.get(usize::try_from(starting_dye).ok()?).copied()?;
    let final_dye = Dye::VALUES.get(usize::try_from(final_dye).ok()?).copied()?;
    
    let meal = chocodye::make_meal(starting_dye, final_dye);
    Some(SnackList::from(meal.as_slice()))
}

#[no_mangle]
pub extern "C" fn request_menu(starting_dye: i32, final_dye: i32, sl: Option<SnackList>, lang: i32) {
    if let Some(starting_dye) = usize::try_from(starting_dye).ok().and_then(|n| Dye::VALUES.get(n)).copied() {
        if let Some(snacks) = sl {
            if let Some(lang) = usize::try_from(lang).ok().and_then(|n| Lang::VALUES.get(n)).copied() {
                let bundle = lang.into_bundle();
                
                let menu = chocodye::make_menu(starting_dye, snacks);
                
                let mut written = String::new();
                
                write!(written, "<p>{}</p><ul>", message!(&bundle, "required-fruits")).unwrap();
                for (snack, count) in snacks.into_iter().filter(|(_, count)| *count > 0) {
                    write!(written, "<li>{}</li>", snack.quantified_name(&bundle, count as u32)).unwrap();
                }
                
                if snacks.is_empty() {
                    write!(written, r#"<li><span class="emph">{}</li></ul>"#, message!(&bundle, "none")).unwrap();
                }
                else {
                    write!(written, "</ul>").unwrap();
                    
                    write!(written, "<p>{}</p><ul>", message!(&bundle, "feed-order")).unwrap();
                    for (snack, count) in menu {
                        write!(written, "<li>{}</li>", snack.quantified_name(&bundle, count as u32)).unwrap();
                    }
                    write!(written, "</ul>").unwrap();
                    
                    if let Some(final_dye) = usize::try_from(final_dye).ok().and_then(|n| Dye::VALUES.get(n)).copied() {
                        let ds = starting_dye.distance(final_dye);
                        let dd = Dye::DEFAULT_CHOCOBO_COLOR.distance(final_dye);
                        
                        if ds > dd {
                            let ss = snacks.sum();
                            let ds = chocodye::make_meal(Dye::DEFAULT_CHOCOBO_COLOR, final_dye).len();
                            
                            write!(written, "<p>{}</p>", message!(&bundle, "han-lemon-note", { "ratio" = format!("{:.1}", 100_f32 * (1_f32 - (ds as f32 / ss as f32))) })).unwrap();
                        }
                    }
                }
                
                #[link(wasm_import_module = "chocoweb")]
                extern "C" {
                    fn update_menu(dom_ptr: *const u8, dom_len: usize);
                }
                
                // SAFETY: trivial
                unsafe { update_menu(written.as_ptr(), written.len()) };
            }
        }
    }
}
