#include "hud.h"
#include "config.h"

#define HUD_INVENTORY_BOX_WIDTH 7
#define HUD_INVENTORY_BOX_HEIGHT 5
#define HUD_INVENTORY_BOX_MID_AT 3

void HUDSystem::item_box(int index, Entity item, int x, int y)
{
	auto color = HSL(0.0f, 0.0f, 1.0f);
	auto h = color.h;
	auto s = color.s;
	auto l = color.l;

	auto mp = AccessMousePosition::get_mouse_position();
	bool selected = (mp.x >= x && mp.x < x + HUD_INVENTORY_BOX_WIDTH && mp.y >= SCREEN_HEIGHT - HUD_INVENTORY_BOX_HEIGHT - y - 1 && mp.y <= SCREEN_HEIGHT - y);
	
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
		if (!AccessWorld_QueryComponent<Item>::has_component(item)) return;
		if (!AccessWorld_QueryComponent<Symbol>::has_component(item)) return;

		auto name = AccessWorld_QueryComponent<Item>::get_component(item).name;
		auto sym = AccessWorld_QueryComponent<Symbol>::get_component(item).sym;
		str({ x + HUD_INVENTORY_BOX_MID_AT, SCREEN_HEIGHT - 4 - y - (int)selected }, sym, color);
		str({ x + HUD_INVENTORY_BOX_MID_AT - 1, SCREEN_HEIGHT - 6 - y - (int)selected }, std::string("[") + std::to_string(index + 1) + "]", color);

		if (selected)
		{
			str({ x, SCREEN_HEIGHT - 8 - y }, name, color);
		}
	}
}

void HUDSystem::activate()
{
	int full_width = INVENTORY_SIZE * HUD_INVENTORY_BOX_WIDTH + (INVENTORY_SIZE - 1);
	int center = (SCREEN_WIDTH - full_width) / 2;

	auto all_players = AccessWorld_QueryAllEntitiesWith<Player, Inventory>::query();
	for (auto&& [ e, inv ] : all_players.each())
	{ 
		for (int i = 0; i < INVENTORY_SIZE; i++)
		{
			item_box(i, inv.stuff[i], center + i * 8);
		}
	}

}