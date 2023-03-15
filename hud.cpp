#include "hud.h"
#include "config.h"

#include <sstream>

#define HUD_INVENTORY_BOX_WIDTH 7
#define HUD_INVENTORY_BOX_HEIGHT 5
#define HUD_INVENTORY_BOX_MID_AT 3

void HUDSystem::label(std::string lab, std::string message, int x, int y, RGB label_color, RGB text_color)
{
	str({ x, y }, lab, label_color);
	str({ x + (int)lab.size(), y }, message, text_color);
}

bool HUDSystem::item_box(int index, Entity item, int x, int y)
{
	auto color = (item == entt::null) ? HSL(0.0f, 0.0f, 1.0f) : HSL(30.0f, 0.5f, 1.0f);
	
	auto h = color.h;
	auto s = color.s;
	auto l = color.l;

	const auto mp = AccessResource_Mouse::get_mouse_position();
	bool selected = (mp.x >= x && mp.x < x + HUD_INVENTORY_BOX_WIDTH && mp.y >= SCREEN_HEIGHT - HUD_INVENTORY_BOX_HEIGHT - y - 1 && mp.y <= SCREEN_HEIGHT - y);
	if (item == entt::null) selected = false;
	
	str({ x, SCREEN_HEIGHT - 6 - y - (int)selected }, "+-----+", color);
	str({ x, SCREEN_HEIGHT - 5 - y - (int)selected }, "|     |", color * 0.8f);
	str({ x, SCREEN_HEIGHT - 4 - y - (int)selected }, "|     |", color * 0.7f);
	str({ x, SCREEN_HEIGHT - 3 - y - (int)selected }, "|     |", color * 0.6f);
	str({ x, SCREEN_HEIGHT - 2 - y - (int)selected }, "+-----+", color * 0.5f);

	str({ x + HUD_INVENTORY_BOX_WIDTH, SCREEN_HEIGHT - 5 - y - (int)selected }, "+", color * 0.4f);
	str({ x + HUD_INVENTORY_BOX_WIDTH, SCREEN_HEIGHT - 4 - y - (int)selected }, "|", color * 0.4f);
	str({ x + HUD_INVENTORY_BOX_WIDTH, SCREEN_HEIGHT - 3 - y - (int)selected }, "|", color * 0.3f);
	str({ x + HUD_INVENTORY_BOX_WIDTH, SCREEN_HEIGHT - 2 - y - (int)selected }, "|", color * 0.3f);
	str({ x + 1, SCREEN_HEIGHT - 1 - y - (int)selected }, "+-----+", color * 0.2f);

	if (item != entt::null)
	{
		if (!AccessWorld_QueryComponent<Item>::has_component(item)) return false;
		if (!AccessWorld_QueryComponent<Symbol>::has_component(item)) return false;

		auto name = AccessWorld_QueryComponent<Item>::get_component(item).name;
		auto sym = AccessWorld_QueryComponent<Symbol>::get_component(item).sym;

		str({ x + HUD_INVENTORY_BOX_MID_AT, SCREEN_HEIGHT - 4 - y - (int)selected }, sym, color);
		str({ x + HUD_INVENTORY_BOX_MID_AT - 1, SCREEN_HEIGHT - 6 - y - (int)selected }, std::string("[") + std::to_string(index + 1) + "]", color);

		if (selected)
		{
			str({ x, SCREEN_HEIGHT - 9 - y }, name, color * 1.5f);
			return true;
		}
	}

	return false;
}

void HUDSystem::activate()
{
	const auto& calendar = AccessWorld_UseUnique<Calendar>::access_unique();

	std::stringstream str;	

	str.str("");
	str << std::setfill('0') << std::setw(2) << calendar.day;
	label("DAY", str.str(), 1, SCREEN_HEIGHT - 2);

	str.str("");
	str << std::setfill('0') << std::setw(2) << calendar.hour;
	label("H", str.str(), 6, SCREEN_HEIGHT - 2);

	str.str("");
	str << std::setfill('0') << std::setw(2) << calendar.minute;
	label("M", str.str(), 9, SCREEN_HEIGHT - 2);

	int full_width = INVENTORY_SIZE * HUD_INVENTORY_BOX_WIDTH + (INVENTORY_SIZE - 1);
	int center = (SCREEN_WIDTH - full_width) / 2;

	auto& game_context = AccessWorld_UseUnique<GameContext>::access_unique();
	bool skip_this_frame = false;

	auto all_players = AccessWorld_QueryAllEntitiesWith<Player, Inventory>::query();
	for (auto&& [ e, inv ] : all_players.each())
	{
		for (int i = 0; i < INVENTORY_SIZE; i++)
		{
			if (item_box(i, inv.stuff[i], center + i * 8) && game_context == GameContext::Game)
			{
				if (left_button())
				{
					game_context = GameContext::Info;
					skip_this_frame = true;
				}
				else if (right_button())
				{
					inv.stuff[i] = entt::null;
				}				
				
				label("LMB", "INSPECT", 15, SCREEN_HEIGHT - 2);
				label("MMB", "USE", 26, SCREEN_HEIGHT - 2);
				label("RMB", "DROP", 33, SCREEN_HEIGHT - 2);
			}
		}
	}

	if (game_context == GameContext::Info)
	{ 
		frame({ 10, 10 }, 40, 20, "#ffffff"_rgb, "#000000"_rgb);
		
		label("LMB", "BACK", 15, SCREEN_HEIGHT - 2);

		if (!skip_this_frame && left_button())
		{			
			game_context = GameContext::Game;
		}
	}
}