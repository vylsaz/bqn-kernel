kk ← "`123456890-=~!@#$%^&*()_+qwertuiop[]QWERTIOP{}asdfghjkl;ASFGHKL:""zxcvbm,./ZXVBM<>? '"
kv ← "˜˘¨⁼⌜´˝∞¯•÷×¬⎉⚇⍟◶⊘⎊⍎⍕⟨⟩√⋆⌽𝕨∊↑∧⊔⊏⊐π←→↙𝕎⍷𝕣⍋⊑⊒⍳⊣⊢⍉𝕤↕𝕗𝕘⊸∘○⟜⋄↖𝕊𝔽𝔾«⌾»·˙⥊𝕩↓∨⌊≡∾≍≠⋈𝕏⍒⌈≢≤≥⇐‿↩"

IsIdent ← {((𝕩≥'0')∧𝕩≤'9')∨((𝕩≥'a')∧𝕩≤'z')∨((𝕩≥'A')∧𝕩≤'Z')∨(𝕩='.')∨𝕩='_'}
GetIdent ← {𝕊 code:
    ident ← (∧` IsIdent)⊸/⌾⌽code
    {{code>○≠𝕩}◶0‿{'•'≡(-1+≠𝕩)⊑code} 𝕩? "•"∾𝕩; 𝕩} ident
}

IsPrefixOf ← ≤○≠◶0‿(⊣≡≠⊸↑)
ToLower ← +⟜(32×1="A["⊸⍋)

{code S pos:
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
        ¬'•'≡⊑ident? ⟨⟩‿pos‿pos
        ;
        ¬⊑'.'∊ident?
            prefix ← ToLower (∨` '_'⊸≠)⊸/1↓ident
            suffixes ← (≠prefix)↓¨prefix⊸IsPrefixOf¨⊸/•listsys
            ⟨ident⊸∾¨suffixes, pos-≠ident, pos⟩
        ;
            ⟨nsname, name⟩ ← (+`·(»⊸<∨`)⌾⌽'.'⊸=)⊸⊔ ident
            ⟨ok, ns⟩ ← {1‿(•BQN 𝕩)}⎊{0‿𝕩} nsname
            {
            ¬ok? ⟨⟩‿pos‿pos
            ;
            ¬6≡•Type ns? ⟨⟩‿pos‿pos
            ;
                keys ← •ns.Keys ns
                prefix ← ToLower (∨` '_'⊸≠)⊸/1↓name
                suffixes ← (≠prefix)↓¨prefix⊸IsPrefixOf¨⊸/keys
                ⟨ident⊸∾¨suffixes, pos-≠ident, pos⟩
            }
        }
    }
}
