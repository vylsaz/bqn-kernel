define([
    'require',
    'base/js/namespace',
    'codemirror/lib/codemirror',
    'codemirror/addon/mode/loadmode',
    'codemirror/addon/mode/simple',
], function(requireJs, Jupyter, CodeMirror) {

    let kk = Array.from('`123456890-=~!@#$%^&*()_+qwertuiop[]QWERTIOP{}asdfghjkl;ASFGHKL:"zxcvbm,./ZXVBM<>? \'');
    let kv = Array.from('˜˘¨⁼⌜´˝∞¯•÷×¬⎉⚇⍟◶⊘⎊⍎⍕⟨⟩√⋆⌽𝕨∊↑∧⊔⊏⊐π←→↙𝕎⍷𝕣⍋⊑⊒⍳⊣⊢⍉𝕤↕𝕗𝕘⊸∘○⟜⋄↖𝕊𝔽𝔾«⌾»·˙⥊𝕩↓∨⌊≡∾≍≠⋈𝕏⍒⌈≢≤≥⇐‿↩');
    let keys = {};
    kk.map((k,i)=>{keys[k] = kv[i];});

    let keymode = 0;
    // 1 for prefix
    let prefix = '\\';
    let modified = (ev)=>ev.shiftKey || ev.ctrlKey || ev.altKey || ev.metaKey;
    let typeChar = (t, c, ev) => {
        let v = t.value;
        let i = t.selectionStart;
        t.value = v.slice(0, i) + c + v.slice(t.selectionEnd);
        t.selectionStart = t.selectionEnd = i + c.length;
        return false;
    }

    function onkeydown(ev) {
        let k = ev.which;
        if (16 <= k && k <= 20) {
            return;
        }
        if (k == 13 && modified(ev)) {
            // *-enter
            return;
        }
        if (keymode) {
            keymode = 0;
            let c = keys[ev.key];
            if (c) 
                return typeChar(ev.target, c, ev);
        } else if (ev.key == prefix) {
            keymode = 1;
            return false;
        }
    }

    function onload() {
        $('head').append(
            $('<link rel="stylesheet" type="text/css" />').attr(
                'href', requireJs.toUrl('./bqn.css')
            )
        );

        $(document).keydown(onkeydown);
        
        // highlighting

        let numb = /(?<![A-Z_a-z0-9π∞¯])¯?(¯_*)?((\d[\d_]*(\.\d[\d_]*)?|π_*)([eE]_*(¯_*)?\d[\d_]*)?|∞_*)(i_*(¯_*)?((\d[\d_]*(\.\d[\d_]*)?|π_*)([eE]_*(¯_*)?\d[\d_]*)?|∞_*))?/u;
        let mod2 = /(‿|\b|^)(•?_[_A-ZÀ-ÖØ-Þa-zß-öø-ÿ0-9π∞¯]*_)(‿|\b)/u;
        let mod1 = /(‿|\b|^)(•?_[_A-ZÀ-ÖØ-Þa-zß-öø-ÿ0-9π∞¯]*)(‿|\b)/u;
        let func = /(‿|\b|^)(•?[A-ZÀ-ÖØ-Þ][_A-ZÀ-ÖØ-Þa-zß-öø-ÿ0-9π∞¯]*)(‿|\b)/u;
        let subj = /(‿|\b|^)(•?[a-zß-öø-ÿ][_A-ZÀ-ÖØ-Þa-zß-öø-ÿ0-9π∞¯]*)(‿|\b)/u;
    
        // can't use "bqn"
        CodeMirror.defineSimpleMode("ibqn", {
            start: [
                {regex: /[\{\[\(⟨]/, indent: true},
                {regex: /[\}\]\)⟩]/, dedent: true},
                {regex: /".*?"/, token: "string"},
                {regex: /'.'/, token: "string"},
                {regex: /@/, token: "string"},
                {regex: /#.*?$/, token: "comment"},
                {regex: numb, token: "number"},
                {regex: /[∘○⊸⟜⌾⊘◶⎊⎉⚇⍟]/u, token: "bqn-mod2"},
                {regex: /_𝕣_/u, token: "bqn-mod2"},
                {regex: mod2, token: [null, "bqn-mod2", null]},
                {regex: /[˙˜˘¨´˝`⌜⁼]/u, token: "bqn-mod1"},
                {regex: /_𝕣/u, token: "bqn-mod1"},
                {regex: mod1, token: [null, "bqn-mod1", null]},
                {regex: /[𝔽𝔾𝕎𝕏𝕊+\-×÷⋆√⌊⌈|¬∧∨<>≠=≤≥≡≢⊣⊢⥊∾≍⋈↑↓↕«»⌽⍉/⍋⍒⊏⊑⊐⊒∊⍷⊔!⍕⍎]/u, token: "bqn-func"},
                {regex: func, token: [null, "bqn-func", null]},
                {regex: /[𝕗𝕘𝕨𝕩𝕤]/u, token: "bqn-keyw"},
                {regex: subj, token: [null, "bqn-subj", null]},
            ]
        });

        CodeMirror.defineMIME('text/bqn', 'ibqn');

        if (Jupyter.notebook.set_codemirror_mode){
            Jupyter.notebook.set_codemirror_mode('ibqn')
        }
    }
    return { onload };
});

