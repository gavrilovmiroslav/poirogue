#include "engine.h"

#include <cstdio>
#include <fstream>
#include <sstream>

PoirogueEngine::PoirogueEngine()
    : engine_running{ true }
{
    PoirogueEngine::Instance = this;

    tcod_console = tcod::Console{ 80, 52 };
    auto params = TCOD_ContextParams{};

    params.tcod_version = TCOD_COMPILEDVERSION;
    params.console = tcod_console.get();
    params.window_title = "Poirogue";
    params.sdl_window_flags = SDL_WINDOW_SHOWN;

    params.vsync = true;
    params.pixel_width = 1200;
    params.pixel_height = 780;

    auto tileset = tcod::load_tilesheet("Taffer_10x10.png", { 16, 16 }, tcod::CHARMAP_CP437);
    
    params.tileset = tileset.get();

    tcod_context = tcod::Context(params);

    SDL_ShowCursor(false);
}

PoirogueEngine::~PoirogueEngine()
{
    SDL_ShowCursor(true);
}

void PoirogueEngine::restart_game()
{
    entt_world.clear();

    for (auto& system : one_offs_systems)
    {
        system->activate();
    }
}

void PoirogueEngine::start_frame()
{
    TCOD_console_clear(tcod_console.get());
}

void PoirogueEngine::poll_events()
{
    SDL_Event event;    
    while (SDL_PollEvent(&event)) {
        tcod_context.convert_event_coordinates(event);
        switch (event.type) {
        case SDL_QUIT:
            engine_running = false;
            break;

        case SDL_KEYUP:
            entt_events.trigger<KeyEvent>(KeyEvent
                {
                    (KeyCode)event.key.keysym.scancode,
                    (event.key.keysym.mod & SDL_Keymod::KMOD_LCTRL) == SDL_Keymod::KMOD_LCTRL,
                    (event.key.keysym.mod & SDL_Keymod::KMOD_LALT) == SDL_Keymod::KMOD_LALT,
                    (event.key.keysym.mod & SDL_Keymod::KMOD_LSHIFT) == SDL_Keymod::KMOD_LSHIFT,
                });
            break;

        case SDL_MOUSEBUTTONUP:
            entt_events.trigger<MouseEvent>(MouseEvent
                {
                    (MouseButton)event.button.button,
                    event.button.x,
                    event.button.y
                });
            break;

        case SDL_MOUSEMOTION:
            mouse_position.x = event.motion.x;
            mouse_position.y = event.motion.y;
            break;

        default:
            entt_events.trigger<WindowEvent>(WindowEvent{ event });
            break;
        }
    }
}

void PoirogueEngine::end_frame()
{
    tcod_context.present(tcod_console);
}

void PoirogueEngine::run_systems()
{
    for (auto& system : runtime_systems)
    {
        system->activate();
    }
}

PoirogueEngine::operator bool() const
{
    return engine_running;
}

void PoirogueEngine::quit()
{
    engine_running = false;
}

PoirogueEngine* PoirogueEngine::Instance = nullptr;
std::unordered_map<size_t, Entity> Access::unique_resources = {};

void AccessConsole::ch(const Position& pt, std::string_view text)
{
    tcod::print(PoirogueEngine::Instance->tcod_console, (std::array<int, 2>&)pt, text, std::nullopt, std::nullopt);
}

void AccessConsole::bg(const Position& pt, RGB color)
{
    std::array<int, 2>& pos = (std::array<int, 2>&)pt;
    if (PoirogueEngine::Instance->tcod_console.in_bounds(pos))
    {
        auto& tile = PoirogueEngine::Instance->tcod_console.at(pos);
        tcod::print(PoirogueEngine::Instance->tcod_console, pos, codepoint_to_utf8(tile.ch), std::nullopt, color);
    }
    else
    {
        tcod::print(PoirogueEngine::Instance->tcod_console, pos, " ", std::nullopt, color);
    }
}

void AccessConsole::fg(const Position& pt, RGB color)
{
    std::array<int, 2>& pos = (std::array<int, 2>&)pt;
    if (PoirogueEngine::Instance->tcod_console.in_bounds(pos))
    {
        auto& tile = PoirogueEngine::Instance->tcod_console.at((std::array<int, 2>&)pt);
        tcod::print(PoirogueEngine::Instance->tcod_console, (std::array<int, 2>&)pt, codepoint_to_utf8(tile.ch), color, std::nullopt);
    }
    else
    {
        tcod::print(PoirogueEngine::Instance->tcod_console, pos, " ", color, std::nullopt);
    }
}