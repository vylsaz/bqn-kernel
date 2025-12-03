{⟨
    rawInput,rawOutput,rawDisplay,rawClear,rawPng, rawWav, 
    isv6, widgetInit, widgetGet, widgetUpdate, widgetDisplay
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

    widgets ← {
        base ← {isv6? 
            ⟨"jupyter-js-widgets", "~2.1.0"⟩;
            ⟨"@jupyter-widgets/base", "2.0.0"⟩
        }
        controls ← {isv6? 
            ⟨"jupyter-js-widgets", "~2.1.0"⟩;
            ⟨"@jupyter-widgets/controls", "2.0.0"⟩
        }
        modelK ← ⟨"_model_module", "_model_module_version", "_model_name"⟩
        viewK ← ⟨"_view_module", "_view_module_version", "_view_name"⟩
        ModelBase ← {modelK≍base∾<𝕩∾"Model"}
        Model ← {modelK≍controls∾<𝕩∾"Model"}
        ViewBase ← {viewK≍base∾<𝕩∾"View"}
        View ← {viewK≍controls∾<𝕩∾"View"}
        WInit ← {{@: !"Cannot init widget"; 𝕩}WidgetInit 𝕩}
        MakeWidget ← {
            id ← WInit 𝕩
            {
                id ⇐ id
                Get ⇐ {{@: !"Unknown attribute"; 𝕩}id WidgetGet 𝕩}
                Update ⇐ {{0: !"Failed to update"; @}id WidgetUpdate 𝕩}
                Display ⇐ {𝕊: WidgetDisplay id}
            }
        }
        And ← ∾⎉1
        Ref ← "IPY_MODEL_"⊸∾
        MkLayout ← {𝕊: WInit (ModelBase"Layout")And ViewBase"Layout"}
        MkStyle ← {WInit (Model 𝕩∾"Style")And ViewBase"Style"}
        Layout ← {[⟨"layout"⟩, ⟨Ref MkLayout 𝕩⟩]}
        Style ← {[⟨"style"⟩, ⟨Ref MkStyle 𝕩⟩]}
        Init ← {(Model 𝕩)And(View 𝕩)And Layout @}

        null  ← <"null"
        true  ← <"true"
        false ← <"false"

        CheckBox ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Init"Checkbox"
            {isv6? @; a And↩ Style"Checkbox", @}
            a And↩ ⍉>⟨
                "description"‿""
                "disabled"‿false
                "value"‿false
            ⟩
            MakeWidget a And 𝕩
        }

        FloatProgress ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Layout @
            a And↩ Model{isv6? "Progress"; "FloatProgress"}
            a And↩ View"Progress"
            a And↩ Style"Progress"
            a And↩ ⍉>⟨
                "bar_style"‿""
                "description"‿""
                "min"‿0.0
                "max"‿1.0
                "orientation"‿"horizontal"
                "value"‿0.0
            ⟩
            MakeWidget a And 𝕩
        }

        FloatSlider ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Init"FloatSlider"
            a And↩ Style"Slider"
            a And↩ ⍉>⟨
                "value"‿0.0
                "min"‿0.0
                "max"‿1.0
                "step"‿0.1
                "description"‿""
                "disabled"‿false
                "continuous_update"‿true
                "orientation"‿"horizontal"
                "readout"‿true,
                "readout_format"‿".2f"
            ⟩
            MakeWidget a And 𝕩
        }

        IntProgress ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Layout @
            a And↩ Model{isv6? "Progress"; "IntProgress"}
            a And↩ View"Progress"
            a And↩ Style"Progress"
            a And↩ ⍉>⟨
                "bar_style"‿""
                "description"‿""
                "min"‿0
                "max"‿100
                "orientation"‿"horizontal"
                "value"‿0
            ⟩
            MakeWidget a And 𝕩
        }

        IntSlider ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Init"IntSlider"
            a And↩ Style"Slider"
            a And↩ ⍉>⟨
                "value"‿0
                "min"‿0
                "max"‿100
                "step"‿1
                "description"‿""
                "disabled"‿false
                "continuous_update"‿true
                "orientation"‿"horizontal"
                "readout"‿true,
                "readout_format"‿"d"
            ⟩
            MakeWidget a And 𝕩
        }

        ToggleButton ⇐ {𝕊@: 𝕊⟨⟩;
            a ← Init"ToggleButton"
            a And↩ Style"ToggleButton"
            a And↩ ⍉>⟨
                "button_style"‿""
                "description"‿""
                "disabled"‿false
                "icon"‿""
                "tooltip"‿null
                "value"‿false
            ⟩
            MakeWidget a And 𝕩
        }
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
                widgets ⇐ widgets
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
        ⟩
    }
}
