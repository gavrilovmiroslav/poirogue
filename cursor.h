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
        auto& mp = AccessMousePosition::get_mouse_position();
        bg(mp, HSL(200.0f, 1.0f, 0.5f));        
    }
};
