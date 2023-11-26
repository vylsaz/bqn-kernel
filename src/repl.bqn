{⟨rawInput,rawOutput,rawDisplay,rawClear⟩:
    Input ← {
        "Prompt 𝕨 must be a string"!⊑(•internal.ElType 𝕨)∊[5,6,7]
        𝕨 RawInput +𝕩
    }
    Output ← {
        "Name 𝕨 must be a string"!⊑(•internal.ElType 𝕨)∊[5,6,7]
        "Text 𝕩 must be a string"!⊑(•internal.ElType 𝕩)∊[5,6,7]
        𝕨 RawOutput 𝕩
    }
    Display ← {
        "Mimetype 𝕨 must be a string"!⊑(•internal.ElType 𝕨)∊[5,6,7]
        "Data 𝕩 must be a string"!⊑(•internal.ElType 𝕩)∊[5,6,7]
        𝕨 RawDisplay 𝕩
    }
    Clear ← {
        0: RawClear 0;
        1: RawClear 1;
        !"Argument must be boolean"
    }

    hasSH ← {⟨"sh"‿(•BQN 𝕩)⟩}⎊⟨⟩"•sh"

    Cfg ← {
    ⟨⟩ 𝕊 mode: ⟨"./",".",⟨⟩⟩ 𝕊 mode;
    ⟨path⟩ 𝕊 mode: ⟨path,".",⟨⟩⟩ 𝕊 mode;
    ⟨path,name⟩ 𝕊 mode: ⟨path,name,⟨⟩⟩ 𝕊 mode;
    ⟨path,name,args⟩ 𝕊 mode:
        repl ⇐ mode

        inherited ← ⟨
            "rebqn"‿•ReBQN
            "primitives"‿•primitives
            "while"‿•_while_
            "wdpath"‿•wdpath
            
            "file"‿•file
            "fchars"‿•FChars
            "flines"‿•FLines
            "fbytes"‿•FBytes

            "parsefloat"‿•ParseFloat
            "repr"‿•Repr
            "fmt"‿•Fmt

            "ffi"‿•FFI

            "type"‿•Type
            "glyph"‿•Glyph
            "decompose"‿•Decompose
            "ns"‿•ns

            "unixtime"‿•UnixTime
            "monotime"‿•MonoTime
            "delay"‿•Delay
            "timed"‿•_timed

            "math"‿•math
            "makerand"‿•MakeRand
            "rand"‿•rand
            "bit"‿•bit
            
            "currenterror"‿•CurrentError
            "fromutf8"‿•FromUTF8
            "toutf8"‿•ToUTF8
            "internal"‿•internal
        ⟩∾hasSH

        modified ← ⟨
            "path"‿path
            "name"‿name
            "args"‿args
            "state"‿⟨path,name,args⟩
            "bqn"‿Exec
            "import"‿{
            𝕊 𝕩: ⟨⟩ 𝕊 𝕩;
            𝕨 𝕊 𝕩:
                file ← •wdpath •file.At 𝕩
                path ← •file.Parent file
                name ← •file.Name file
                ⟨path,name,𝕨⟩ Exec •FChars file
            }
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

        known ← inherited∾modified 

        system ⇐ known∾⟨"listsys"‿((0⊑¨known)∾<"listsys")⟩
    }

    Exec ← {
    𝕊 𝕩: ⟨⟩ 𝕊 𝕩;
    𝕨 𝕊 𝕩:
        r ← •ReBQN 𝕨 Cfg "none"
        R 𝕩
    }

    r ← •ReBQN ⟨⟩ Cfg "loose"
    {1‿(•Fmt R 𝕩)}⎊{0‿(•CurrentError 𝕩)}
}
