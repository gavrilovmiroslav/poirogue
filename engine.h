#pragma once

#include <SDL2/SDL.h>
#include <libtcod.h>
#include <entt/entt.hpp>

#include <unordered_map>
#include <vector>

#include "utils.h"
#include "common.h"

struct System 
{
    virtual void activate() {}
};

struct OneOffSystem : public System 
{
    virtual void activate() {}
};

struct RuntimeSystem : public System 
{
    virtual void activate() {}
};

struct PoirogueEngine final
{
	PoirogueEngine();
	~PoirogueEngine();
	
	void restart_game();
	void start_frame();
    void poll_events();
	void end_frame();
	void run_systems();
	
	operator bool() const;
	
	void quit();

protected:
	std::vector<std::shared_ptr<OneOffSystem>> one_offs_systems;
	std::vector<std::shared_ptr<RuntimeSystem>> runtime_systems;

public:
    template<typename T>
    std::shared_ptr<T> add_one_off_system()
    {
        std::shared_ptr<T> ptr{ new T };
        one_offs_systems.push_back(ptr);

        return ptr;
    }

    template<typename T>
    std::shared_ptr<T> add_runtime_system()
    {
        std::shared_ptr<T> ptr{ new T };
        runtime_systems.push_back(ptr);

        return ptr;
    }

protected:
	ecs::registry entt_world;
	ecs::dispatcher entt_events;

	tcod::Console tcod_console;
	tcod::Context tcod_context;

	static PoirogueEngine* Instance;

private:
    bool mouse_buttons[4] { false, false, false, false };

    ScreenPosition mouse_position;
	bool engine_running;
    
    friend struct AccessConsole;

    friend struct AccessWorld_CheckValidity;
    friend struct AccessResource_Mouse;

    template<typename T>
    friend struct AccessWorld_UseUnique;
    
	friend struct AccessWorld_ModifyWorld;
	friend struct AccessWorld_ModifyEntity;

    template<typename T>
    friend struct AccessWorld_QueryComponent;

    template<typename... Qs>
    friend struct AccessWorld_QueryAllEntitiesWith;

	template<typename T>
	friend struct AccessEvents_Emit;

	template<typename T>
	friend struct AccessEvents_Listen;

	template<typename T>
	friend struct ScriptComponent;

    friend struct AccessWorld_DirectRegistry;
};

struct Access
{};

struct AccessWorld_DirectRegistry : public Access
{
    entt::registry& get_registry()
    {
        return PoirogueEngine::Instance->entt_world;
    }
};

namespace YAML
{
    class Node;
}

struct AccessYAML : public Access
{
    YAML::Node load(const char* name);
};

struct AccessConsole : public Access
{
    void box(const ScreenPosition& pt, int w, int h, RGB fg, RGB bg, char c = ' ');
    void frame(const ScreenPosition& pt, int w, int h, RGB fg, RGB bg);
    void str(const ScreenPosition& pt, std::string_view text, RGB fg);
    void ch(const ScreenPosition& pt, std::string_view text);
    void bg(const ScreenPosition& pt, RGB color);
    void fg(const ScreenPosition& pt, RGB color);
};

struct AccessWorld_CheckValidity : public Access
{
    bool is_valid(Entity entity) const
    {
        return PoirogueEngine::Instance->entt_world.valid(entity);
    }
};

struct AccessResource_Mouse : public Access
{
    const bool left_button() const
    {
        return PoirogueEngine::Instance->mouse_buttons[1];
    }

    const bool mid_button() const
    {
        return PoirogueEngine::Instance->mouse_buttons[2];
    }

    const bool right_button() const
    {
        return PoirogueEngine::Instance->mouse_buttons[3];
    }

    const ScreenPosition& get_mouse_position() const
    {
        return PoirogueEngine::Instance->mouse_position;
    }
};

template<typename T>
T& get_res()
{
    static T unique_resource;
    return unique_resource;
}

template<typename T>
struct AccessWorld_UseUnique : public Access
{
    T& access_unique()
    {        
        return get_res<T>();
    }
};

struct AccessWorld_ModifyWorld : public Access
{
    Entity create_entity();

    void destroy_entity(Entity entity)
	{
        if (PoirogueEngine::Instance->entt_world.valid(entity))
    		PoirogueEngine::Instance->entt_world.destroy(entity);
	}

    template<typename It>
    void destroy_entities(It start, It end)
    {
        PoirogueEngine::Instance->entt_world.destroy(start, end);
    }
};

struct AccessWorld_ModifyEntity : public Access
{

    template<typename T>
    void add_tag_component(Entity entity)
    {
        PoirogueEngine::Instance->entt_world.emplace<T>(entity);
    }
    
    template<typename T, typename... Args>
	T& add_component(Entity entity, Args&&... args)
	{        
		return PoirogueEngine::Instance->entt_world.emplace_or_replace<T>(entity, args...);
	}

	template<typename T>
	void remove_component(Entity entity)
	{
		PoirogueEngine::Instance->entt_world.remove<T>(entity);
	}
};

template<typename T>
struct AccessWorld_QueryComponent : public Access
{    
    bool has_component(Entity e)
    {
        return PoirogueEngine::Instance->entt_world.all_of<T>(e);
    }
    
    T& get_component(Entity e)
    {
        return PoirogueEngine::Instance->entt_world.get<T>(e);
    }
};

template<typename... Qs>
struct AccessWorld_QueryAllEntitiesWith : public Access
{
	auto query()
	{
		return PoirogueEngine::Instance->entt_world.view<Qs...>();
	}
};

template<typename T>
struct AccessEvents_Emit : public Access
{
    void emit_event()
    {
        PoirogueEngine::Instance->entt_events.trigger<T>();
    }

    void emit_event(T signal)
    {
        PoirogueEngine::Instance->entt_events.trigger<T>(std::forward<T>(signal));
    }
};

template<typename T>
struct AccessEvents_Listen : public Access
{
	AccessEvents_Listen()
	{
		PoirogueEngine::Instance->entt_events.sink<T>().connect<&AccessEvents_Listen<T>::react_to_event>(this);
	}

	virtual void react_to_event(T& signal) = 0;
};

template<int T>
struct AccessTick 
    : public AccessEvents_Listen<Tick>
{
    int tick_time = T;
    int current_time = T;

    void react_to_event(Tick&) override
    {
        current_time++;
        if (current_time >= tick_time)
        {
            tick();
            current_time = 0;
        }
    }

    virtual void tick() = 0;
};
