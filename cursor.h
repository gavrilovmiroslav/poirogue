#pragma once

#include "common.h"
#include "engine.h"

struct MouseCursorSystem
    : public RuntimeSystem
    , public AccessMousePosition
    , public AccessConsole
{
    void activate() override
    {
        static float dsat = 0.0f;

        auto& mp = AccessMousePosition::get_mouse_position();

        float hue = 200.0f;
        float sat = 0.0f;
        float val = 1.0f;
        
        bg({ mp.x, mp.y }, HSL(hue, sat, val));
    }
};
