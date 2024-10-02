kk ← "`123456890-=~!@#$%^&*()_+qwertuiop[]QWERTIOP{}asdfghjkl;ASFGHKL:""zxcvbm,./ZXVBM<>? '"
kv ← "˜˘¨⁼⌜´˝∞¯•÷×¬⎉⚇⍟◶⊘⎊⍎⍕⟨⟩√⋆⌽𝕨∊↑∧⊔⊏⊐π←→↙𝕎⍷𝕣⍋⊑⊒⍳⊣⊢⍉𝕤↕𝕗𝕘⊸∘○⟜⋄↖𝕊𝔽𝔾«⌾»·˙⥊𝕩↓∨⌊≡∾≍≠⋈𝕏⍒⌈≢≤≥⇐‿↩"

IsIdent ← {((𝕩≥'0')∧𝕩≤'9')∨((𝕩≥'a')∧𝕩≤'z')∨((𝕩≥'A')∧𝕩≤'Z')∨(𝕩='.')∨𝕩='_'}
GetIdent ← {𝕊 code:
    ident ← (∧` IsIdent)⊸/⌾⌽code
    {{code>○≠𝕩}◶0‿{'•'≡(-1+≠𝕩)⊑code} 𝕩? "•"∾𝕩; 𝕩} ident
}

IsPrefixOf ← ≤○≠◶0‿(⊣≡≠⊸↑)
ToLower ← +⟜(32×1="A["⊸⍋)

{𝕊 repl:
    {code 𝕊 pos:
        code ↩ pos↑code 
        {
        {𝕊: 1<≠code}◶0‿{𝕊: '\'≡¯2⊑code} @?
            k ← ¯1⊑code
            v ← ⥊(⊑kk⊐k)⊑kv∾k
            ⟨⟨v⟩, pos-2, pos⟩
        ;
            ident ← GetIdent code
            { 
            0≡≠ident? ⟨⟩‿pos‿pos
            ;
            ¬⊑'.'∊ident? 
                {
                ¬'•'≡⊑ident? ⟨⟩‿pos‿pos
                ;
                    name ← 1↓ident
                    prefix ← ToLower (∨` '_'⊸≠)⊸/name
                    suffixes ← (≠prefix)↓¨prefix⊸IsPrefixOf¨⊸/•listsys
                    ⟨name⊸∾¨suffixes, pos-≠name, pos⟩
                }
            ;
                ⟨nsname, dname⟩ ← (+`·(»⊸<∨`)⌾⌽'.'⊸=)⊸⊔ ident
                ⟨ok, ns⟩ ← {1‿(Repl 𝕩)}⎊{0‿𝕩} nsname
                {
                ¬ok? ⟨⟩‿pos‿pos
                ;
                ¬6≡•Type ns? ⟨⟩‿pos‿pos
                ;
                    name ← 1↓dname
                    keys ← •ns.Keys ns
                    prefix ← ToLower (∨` '_'⊸≠)⊸/name
                    suffixes ← (≠prefix)↓¨prefix⊸IsPrefixOf¨⊸/keys
                    ⟨name⊸∾¨suffixes, pos-≠name, pos⟩
                }
            }
        }
    }
}
