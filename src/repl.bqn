{⟨
    rawInput,rawOutput,rawDisplay,rawClear,rawPng, rawWav, 
    widgetTest, widgetGet
⟩:
    IsString ← {(0≡•Type 𝕩)∧⊑(•internal.ElType 𝕩)∊[5,6,7]}
    IsNumericArray ← {(0≡•Type 𝕩)∧(•internal.ElType 𝕩)<5}

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
    Png ← {𝕊 x:
        "Image must be a numeric array"!IsNumericArray x
        r ← =x
        "Rank of image must be 2 or 3"!∨´r=2‿3
        ⟨h,w,c⟩ ← ≢i ← {r≡2? ⍉≍x; x}
        "Color channels of image must be between 1 and 4"!∨´c=1‿2‿3‿4
        ⟨ok, s⟩ ← RawPng i 
        s!ok, s
    }
    Wav ← {w 𝕊 x:
        "Sample rate must be a number"!1≡•Type w
        "Sound must be a numeric array"!IsNumericArray x
        r ← =x
        "Rank of sound must be 1 or 2"!∨´r=1‿2
        a ← ⍉{r≡1? ≍x; x}
        ⟨ok, s⟩ ← w RawWav a
        s!ok, s
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
                Image ⇐ {
                    p ← Png 𝕩
                    h ← "<div><img style=""image-rendering:pixelated;"" src=""data:image/png;base64,"∾p∾""" /></div>"
                    HTML h, @
                }
                audioFreq ⇐ 44100
                Audio ⇐ {𝕊 𝕩: audioFreq 𝕊 𝕩;
                    w ← 𝕨 Wav 𝕩
                    h ← "<div><audio controls src=""data:audio/wav;base64,"∾w∾""" /></div>"
                    HTML h, @
                }
            }
            "platform"‿{
                os ⇐ •platform.os
                bqn ⇐ •platform.bqn
                cpu ⇐ •platform.cpu
                environment ⇐ "jupyter"
            }
            "exit"‿{𝕊: 
                msg ← "•Exit is disabled for jupyter notebook. "
                !msg∾"If in console, use )off to exit"
            }
            "test"‿widgetTest
            "get"‿widgetGet
        ⟩
    }
}
