
const imports = {
    chocoweb: {
        update_menu: updateOutput
    }
};

WebAssembly.instantiateStreaming(fetch("../target/wasm32-unknown-unknown/release/chocoweb.wasm"), imports).then(
    (result) => {
        wasm  = result.instance
        
        updateLang(read("lang-select"))
    },
);

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
        wasm.exports.request_menu(start, meal, lang)
    }
}
