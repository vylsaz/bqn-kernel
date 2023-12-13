{⟨rawInput,rawOutput,rawDisplay,rawClear⟩:
    IsString ← {
        (0≡•Type 𝕩)∧⊑(•internal.ElType 𝕩)∊[5,6,7]
    }

    Input ← {
        "Prompt must be a string"!IsString 𝕨
        𝕨 RawInput +𝕩
    }
    Output ← {
        "Name 𝕨 must be a string"!IsString 𝕨
        "Text 𝕩 must be a string"!IsString 𝕩
        𝕨 RawOutput 𝕩
    }
    Display ← {
        "Mimetype 𝕨 must be a string"!IsString 𝕨
        "Data 𝕩 must be a string"!IsString 𝕩
        𝕨 RawDisplay 𝕩
    }
    Clear ← {
        0: RawClear 0;
        1: RawClear 1;
        !"Argument must be boolean"
    }

    •ReBQN {
        repl ⇐ "loose"

        system ⇐ {𝕩∾•listSys(¬∘∊/⊣)⊑¨𝕩}⟨
            "out"‿{"stdout" Output 𝕩}
            "show"‿{"stdout" Output •Fmt 𝕩, 𝕩}
            "getline"‿{𝕊: "" Input 0}
            "jupyter"‿{
                Out ⇐ {"stdout" Output 𝕩}
                Err ⇐ {"stderr" Output 𝕩}
                GetInput    ⇐ {𝕩 Input 0}
                GetPassword ⇐ {𝕩 Input 1}
                HTML ⇐ {"text/html"  Display 𝕩}
                Text ⇐ {"text/plain" Display 𝕩}
                Display ⇐ Display
                Clear ⇐ Clear
            }
            "platform"‿{
                os ⇐ •platform.os
                bqn ⇐ •platform.bqn
                cpu ⇐ •platform.cpu
                environment ⇐ "jupyter"
            }
        ⟩
    }
}
