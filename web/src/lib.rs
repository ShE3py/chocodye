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

fn cvt(dye: i32) -> Option<Dye> {
    Dye::VALUES.get(usize::try_from(dye).ok()?).copied()
}

#[no_mangle]
pub extern "C" fn make_meal(starting_dye: i32, final_dye: i32) -> Option<SnackList> {
    let starting_dye = cvt(starting_dye)?;
    let final_dye = cvt(final_dye)?;

    let meal = chocodye::make_meal(starting_dye, final_dye);
    Some(SnackList::from(meal.as_slice()))
}

#[no_mangle]
pub extern "C" fn request_menu(starting_dye: i32, final_dye: i32, sl: Option<SnackList>, lang: i32) {
    let Some(((starting_dye, final_dye), snacks)) = cvt(starting_dye).zip(cvt(final_dye)).zip(sl) else {
        return;
    };
    let Some(bundle) = usize::try_from(lang).ok().and_then(|n| Lang::VALUES.get(n)).copied().map(Lang::into_bundle) else {
        return;
    };

    let menu = chocodye::make_menu(starting_dye, snacks);

    let mut written = String::new();

    _ = write!(written, "<p>{}</p><ul>", message!(&bundle, "required-fruits"));
    for (snack, count) in snacks.into_iter().filter(|(_, count)| *count > 0) {
        _ = write!(written, "<li>{}</li>", snack.quantified_name(&bundle, count as u32));
    }
    if snacks.is_empty() {
        _ = write!(written, r#"<li><span class="emph">{}</li></ul>"#, message!(&bundle, "none"));
    }

    else {
        _ = write!(written, "</ul>");

        _ = write!(written, "<p>{}</p><ul>", message!(&bundle, "feed-order"));
        for (snack, count) in menu {
            _ = write!(written, "<li>{}</li>", snack.quantified_name(&bundle, count as u32));
        }
        _ = write!(written, "</ul>");

        let ds = starting_dye.distance(final_dye);
        let dd = Dye::DEFAULT_CHOCOBO_COLOR.distance(final_dye);

        #[expect(clippy::cast_precision_loss, reason = "precision not required as we're rounding to .1")]
        if ds > dd {
            let ss = snacks.sum();
            let ds = chocodye::make_meal(Dye::DEFAULT_CHOCOBO_COLOR, final_dye).len();

            _ = write!(written, "<p>{}</p>", message!(&bundle, "han-lemon-note", { "ratio" = format!("{:.1}", 100_f32 * (1_f32 - (ds as f32 / ss as f32))) }));
        }
    }

    #[link(wasm_import_module = "chocoweb")]
    unsafe extern "C" {
        fn update_menu(dom_ptr: *const u8, dom_len: usize);
    }

    // SAFETY: trivial
    unsafe { update_menu(written.as_ptr(), written.len()) };
}
