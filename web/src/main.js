
const imports = {
    chocoweb: {
        update_menu: updateOutput
    }
};

// https://stackoverflow.com/a/47880734
// https://github.com/GoogleChromeLabs/wasm-feature-detect
const supported = (() => {
    try {
        if(typeof WebAssembly === "object" && typeof WebAssembly.instantiateStreaming === "function") {
            const module = new WebAssembly.Module(new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96 , 0, 0, 3, 2, 1, 0, 10, 8, 1, 6, 0, 65, 0, 192, 26, 11]));
            if(module instanceof WebAssembly.Module) {
                return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
            }
        }
    }
    catch(ignored) {}
    
    return false;
})();

if(supported) {
    WebAssembly.instantiateStreaming(fetch("chocoweb.wasm"), imports).then(
        (result) => {
            wasm = result.instance
            
            let lang = read("lang-select")
            
            try {
                let _lang = parseInt(document.cookie.split("; ").find((row) => row.startsWith("lang=")).split("=")[1])
                
                if(_lang >= 0 && _lang < 4 && _lang !== lang) {
                    restore("lang-select", _lang)
                    lang = _lang
                }
            }
            catch(ignored) {}
            
            updateLang(lang)
        },
    );
}
else {
    document.getElementById("lang_import").innerHTML = "&#x2718; Outdated web browser; <a href=\"https://webassembly.org/roadmap/\" target=\"_blank\">WebAssembly with sign-extension operators required</a>."
    document.getElementById("lang-select").hidden = true
}

function readUsize(ptr) {
    return new DataView(wasm.exports.memory.buffer).getUint32(ptr, true)
}

function readStr(ptr, len) {
    return (new TextDecoder()).decode(new Uint8Array(wasm.exports.memory.buffer,  ptr, len))
}

function read(select_id) {
    let select = document.getElementById(select_id)
    
    if(select) {
        return select.options[select.selectedIndex].value
    }
}

function restore(select_id, saved) {
    let select = document.getElementById(select_id)
    
    if(select && saved) {
        select.value = saved
    }
}

function updateLang(lang) {
    console.assert(lang >= 0 && lang < 4)
    
    if(lang >= 0 && lang < 4) {
        let start = read("start-select")
        let final = read("final-select")
        
        document.getElementById("lang_import").innerHTML = readStr(readUsize(wasm.exports.LANGS.value + 8 * lang), readUsize(wasm.exports.LANG_SIZES.value  + 4 * lang))
        
        restore("start-select", start)
        restore("final-select", final)
        
        calculate()
        
        document.documentElement.lang = ["en", "fr", "de", "jp"][lang]
        document.cookie = "lang=" + lang + "; Max-Age=15552000; SameSite=Strict; Secure"
    }
}

function updateOutput(outputPtr, outputLen) {
    document.getElementById("output_import").innerHTML = readStr(outputPtr, outputLen)
}

function calculate() {
    let start = read("start-select")
    let final = read("final-select")
    let lang = read("lang-select")
    
    if(start && final && lang) {
        let meal = wasm.exports.make_meal(start, final)
        wasm.exports.request_menu(start, final, meal, lang)
    }
}
