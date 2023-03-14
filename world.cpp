#include "world.h"
#include "common.h"
#include "level.h"

void WorldCrafting::execute_crafting()
{
	auto& level = AccessWorld_UseUnique<Level>::access_unique();
	auto& people_mapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();

	for (int i = 0; i < REGION_COUNT; i++)
	{
		const auto place_entity = people_mapping.places[i];
		const auto name = people_mapping.graph->get_node_label(place_entity);

		auto tiles = level.region_tiles[i];
		auto center = level.region_centers[i];

		if (name == "WAREHOUSE")
			create_warehouse(level, people_mapping, i, tiles, center);
		else if (name == "MACHINE SHOP")
			create_machine_shop(level, people_mapping, i, tiles, center);
		else if (name == "LIBRARY")
			create_library(level, people_mapping, i, tiles, center);
		else if (name == "CLERIC")
			create_cleric(level, people_mapping, i, tiles, center);
		else if (name == "INN")
			create_inn(level, people_mapping, i, tiles, center);
		else if (name == "TEMPLE")
			create_temple(level, people_mapping, i, tiles, center);
		else if (name == "MESS HALL")
			create_mess_hall(level, people_mapping, i, tiles, center);
		else if (name == "GYMNASIUM")
			create_gymnasium(level, people_mapping, i, tiles, center);
		else if (name == "MINESHAFT")
			create_mineshaft(level, people_mapping, i, tiles, center);
		else if (name == "JUNKYARD")
			create_junkyard(level, people_mapping, i, tiles, center);
		else if (name == "FOUNDRY")
			create_foundry(level, people_mapping, i, tiles, center);
		else if (name == "SKYPORT")
			create_skyport(level, people_mapping, i, tiles, center);
		else if (name == "LOUNGE")
			create_lounge(level, people_mapping, i, tiles, center);
		else if (name == "MARKET")
			create_market(level, people_mapping, i, tiles, center);
		else if (name == "DRINKING HOLE")
			create_drinking_hole(level, people_mapping, i, tiles, center);
		else if (name == "SHRINE")
			create_shrine(level, people_mapping, i, tiles, center);
		else if (name == "ABANDONED WAREHOUSE")
			create_abandoned_warehouse(level, people_mapping, i, tiles, center);
		else if (name == "BLACK MARKET")
			create_black_market(level, people_mapping, i, tiles, center);
		else if (name == "MONOLITH")
			create_monolith(level, people_mapping, i, tiles, center);
		else if (name == "HIDDEN NOOK")
			create_hidden_nook(level, people_mapping, i, tiles, center);
		else if (name == "HAUNTED SPOT")
			create_haunted_spot(level, people_mapping, i, tiles, center);
	}
}

Entity WorldCrafting::create_wares(WorldPosition tile, char sym)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto entity = create_entity();
	std::string s(1, sym);
	add_component<Symbol>(entity, s);
	auto& wp = add_component<WorldPosition>(entity);
	wp.x = tile.x;
	wp.y = tile.y;

	block_sight_walking(entity, wp);
	add_component<Weight>(entity, (int)sym * 5);

	add_component<Colored>(entity, HSL(rng->getFloat(-180.0f, -160.0f), rng->getFloat(0.5f, 0.75f), rng->getFloat(0.5f, 0.85f)));
	add_component<BumpDefault>(entity, CommandType::Inspect, Command{});

	return entity;
}

Entity WorldCrafting::create_machine(WorldPosition tile)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto entity = create_entity();
	std::string s(1, MACHINE_SYM);
	add_component<Symbol>(entity, s);
	auto& wp = add_component<WorldPosition>(entity);
	wp.x = tile.x;
	wp.y = tile.y;

	block_sight_walking(entity, wp);
	add_component<Weight>(entity, (int)MACHINE_SYM * 5);

	add_component<Colored>(entity, HSL(rng->getFloat(-180.0f, -160.0f), rng->getFloat(0.5f, 0.75f), rng->getFloat(0.5f, 0.85f)));
	add_component<BumpDefault>(entity, CommandType::Inspect, Command{});

	return entity;
}

Entity WorldCrafting::create_bookshelf(WorldPosition tile)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto entity = create_entity();
	std::string s(1, BOOKCASE_SYM);
	add_component<Symbol>(entity, s);
	auto& wp = add_component<WorldPosition>(entity);
	wp.x = tile.x;
	wp.y = tile.y;

	block_sight_walking(entity, wp);
	add_component<Weight>(entity, (int)BOOKCASE_SYM * 5);

	// todo: add contents

	add_component<Colored>(entity, HSL(rng->getFloat(15.0f, 30.0f), rng->getFloat(0.5f, 0.75f), rng->getFloat(0.5f, 0.85f)));
	add_component<BumpDefault>(entity, CommandType::Inspect, Command{});

	return entity;
}

Entity WorldCrafting::create_furnace(WorldPosition tile)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto entity = create_entity();
	std::string s(1, FURNACE_SYM);
	add_component<Symbol>(entity, s);
	auto& wp = add_component<WorldPosition>(entity);
	wp.x = tile.x;
	wp.y = tile.y;

	block_sight_walking(entity, wp);
	add_component<Weight>(entity, (int)FURNACE_SYM * 5);

	add_component<BumpDefault>(entity, CommandType::Inspect, Command{});
	add_component<Colored>(entity, HSL(rng->getFloat(-15.0f, 15.0f), 1.0f, 1.0f));
	add_tag_component<Hot>(entity);
	return entity;
}

Entity WorldCrafting::create_chest(WorldPosition tile)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto entity = create_entity();
	std::string s(1, CHEST_SYM);
	add_component<Symbol>(entity, s);
	auto& wp = add_component<WorldPosition>(entity);
	wp.x = tile.x;
	wp.y = tile.y;
	
	block_walking(entity, wp);
	add_component<Weight>(entity, (int)CHEST_SYM * 5);

	// todo: add contents

	if (rng->getInt(0, 100) > 75)
	{
		add_tag_component<Locked>(entity);

		Command comm;
		comm.unlock.chance = rng->getInt(0, 4) * 25;
		add_component<BumpDefault>(entity, CommandType::Unlock, comm);
	}
	else
	{
		add_component<BumpDefault>(entity, CommandType::Inspect, Command{});
	}

	add_component<Colored>(entity, HSL(rng->getFloat(15.0f, 30.0f), rng->getFloat(0.5f, 0.75f), rng->getFloat(0.5f, 0.85f)));

	return entity;
}

void WorldCrafting::block_sight(Entity e, WorldPosition wp)
{
	add_tag_component<Blocked>(e);
	AccessWorld_UseUnique<Level>::access_unique().map->setProperties(wp.x, wp.y, false, true);
}

void WorldCrafting::block_walking(Entity e, WorldPosition wp)
{
	add_tag_component<Blocked>(e);
	AccessWorld_UseUnique<Level>::access_unique().map->setProperties(wp.x, wp.y, true, false);
}

void WorldCrafting::block_sight_walking(Entity e, WorldPosition wp)
{
	add_tag_component<Blocked>(e);
	AccessWorld_UseUnique<Level>::access_unique().map->setProperties(wp.x, wp.y, false, false);
}

void WorldCrafting::create_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	static constexpr int sym_size = 2;
	static char syms[sym_size]{ WIRE_SYM, CHEST_SYM };

	for (auto tile : tiles)
	{
		level.dig[tile.x][tile.y] = FLOOR_SYM;
		level.hues[tile.x][tile.y] = 200.0f;
		level.sats[tile.x][tile.y] = rng->getFloat(0.1f, 0.25f);
	}

	std::sort(tiles.begin(), tiles.end(), [&](WorldPosition& a, WorldPosition& b) { return a.distance(center) > b.distance(center); });

	for (int i = 0; i < tiles.size() / 3; i++)
	{
		if (rng->getInt(0, 100) > 50) continue;

		auto tile = tiles.back();
		tiles.pop_back();

		switch(syms[rng->getInt(0, sym_size - 1)])
		{ 
		case CHEST_SYM:
			create_chest(tile); break;
		case WIRE_SYM:
			create_wares(tile); break;
		}
	}
}

void WorldCrafting::create_machine_shop(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	static constexpr int sym_size = 12;
	static char syms[sym_size]{ WIRE_SYM, WIRE_SYM, WIRE_SYM, MACHINE_SYM, MACHINE_SYM, MACHINE_SYM, MACHINE_SYM, DRILL_SYM, DRILL_SYM, DRILL_SYM, DRILL_SYM, BOOKCASE_SYM };

	std::sort(tiles.begin(), tiles.end(), [&](WorldPosition& a, WorldPosition& b) { return a.distance(center) < b.distance(center); });

	for (auto tile : tiles)
	{
		level.hues[tile.x][tile.y] = rng->getFloat(-180.0f, -160.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.25f, 0.75f);
	}

	for (int i = 0; i < tiles.size() / 2;)
	{
		if (rng->getInt(0, 100) > 50) continue;

		auto tile = tiles.back();
		tiles.pop_back();

		switch (syms[rng->getInt(0, sym_size - 1)])
		{
		case MACHINE_SYM:
			create_machine(tile); break;
		case WIRE_SYM:
			create_wares(tile); break;
		case DRILL_SYM:
			create_wares(tile, DRILL_SYM); break;
		case BOOKCASE_SYM:
			create_bookshelf(tile); break;
		}

		i++;
	}
}

void WorldCrafting::create_library(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	int min_x = 1000;
	int min_y = 1000;
	int max_x = 0;
	int max_y = 0;

	for (auto tile : tiles)
	{
		if (tile.x < min_x) min_x = tile.x;
		if (tile.y < min_y) min_y = tile.y;
		if (tile.x > max_x) max_x = tile.x;
		if (tile.y > max_y) max_y = tile.y;

		level.dig[tile.x][tile.y] = FLOOR_SYM;
		level.hues[tile.x][tile.y] = 0.0f;
		level.sats[tile.x][tile.y] = rng->getFloat(0.4f, 0.75f);
	}

	for (int i = min_x; i <= max_x; i++)
	{
		for (int j = min_y; j <= max_y; j++)
		{
			auto tile = WorldPosition{ i, j };
			if (i % 2 == 0) continue;
			if (std::find(tiles.begin(), tiles.end(), tile) != tiles.end())
			{
				if (rng->getInt(0, 100) > 80) continue;
				create_bookshelf(tile);
			}
		}
	}
}

void WorldCrafting::create_cleric(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_inn(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
}

void WorldCrafting::create_temple(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_mess_hall(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_gymnasium(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_mineshaft(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_junkyard(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	std::vector<WorldPosition> heaps;
	shuffle(tiles);

	for (auto t : tiles)
	{		
		bool okay = true;
		for (auto h : heaps)
		{
			if (t.distance(h) <= 3)
			{
				okay = false;
				break;
			}
		}

		if (okay)
		{
			heaps.push_back(t);			
		}
	}

	for (auto h : heaps)
	{
		for (int i = -1; i < 2; i++)
		{
			for (int j = -1; j < 2; j++)
			{
				auto heap = WorldPosition{ h.x + i, h.y + j };
				if (rng->getInt(0, 100) > 45) continue;

				switch (rng->getInt(0, 13))
				{
				case 0: case 1: case 2: case 3:case 4:case 5:
					create_machine(heap);
					break;
				case 6: case 7: case 8: case 9: case 10: case 11: case 12:
				{
					auto ware = create_wares(heap, 'Z' + rng->getInt(1, 12));
					add_component<Colored>(ware, HSL(rng->getFloat(0.0f, 360.0f), rng->getFloat(0.15f, 0.35f), 1.0f));
					remove_component<Blocked>(ware);
					add_tag_component<Exhausting>(ware);

					break;
				}
				case 13:
					create_chest(heap);
					break;
				}
			}
		}
	}
}

void WorldCrafting::create_foundry(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	int min_x = 1000;
	int min_y = 1000;
	int max_x = 0;
	int max_y = 0;

	for (auto tile : tiles)
	{
		level.hues[tile.x][tile.y] = rng->getFloat(-20.0f, 20.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.9f, 1.0f);
		level.vals[tile.x][tile.y] = 1.0f;

		if (tile.x < min_x) min_x = tile.x;
		if (tile.y < min_y) min_y = tile.y;
		if (tile.x > max_x) max_x = tile.x;
		if (tile.y > max_y) max_y = tile.y;

		level.dig[tile.x][tile.y] = rng->getInt('v', 'y');

		auto hot_air = create_entity();
		add_component<WorldPosition>(hot_air);
		add_tag_component<Hot>(hot_air);
	}

	for (int i = min_x; i <= max_x; i++)
	{
		for (int j = min_y; j <= max_y; j++)
		{
			auto tile = WorldPosition{ i, j };
			if (i % 3 == 0 && j % 3 == 0)
			{
				if (std::find(tiles.begin(), tiles.end(), tile) != tiles.end())
				{
					if (rng->getInt(0, 100) > 60) continue;
					create_furnace(tile);
				}
			}
		}
	}
}

void WorldCrafting::create_skyport(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	int min_x = 1000;
	int min_y = 1000;
	int max_x = 0;
	int max_y = 0;

	for (auto tile : tiles)
	{
		level.hues[tile.x][tile.y] = rng->getFloat(220.0f, 240.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.0f, 0.1f);
		level.vals[tile.x][tile.y] = 1.0f;

		if (tile.x < min_x) min_x = tile.x;
		if (tile.y < min_y) min_y = tile.y;
		if (tile.x > max_x) max_x = tile.x;
		if (tile.y > max_y) max_y = tile.y;
	}

	int center_x = (min_x + max_x) / 2;
	int center_y = (min_y + max_y) / 2;
	center = WorldPosition{ center_x, center_y };
	auto max_distance = center.distance(WorldPosition{ min_x, max_x });

	char syms[4]{ 'i', 'j', 'k', 'l' };
	int index = 0;
	for (int i = min_x; i <= max_x; i++)
	{
		for (int j = min_y; j <= max_y; j++)
		{
			auto tile = WorldPosition{ i, j };
			auto distance = center.distance(tile);

			if (distance < max_distance / 4) continue;
			if (std::find(tiles.begin(), tiles.end(), tile) != tiles.end())
			{
				if (rng->getFloat(0.0f, 1.0f) > (distance / max_distance))
				{
					auto floor_tiling = create_entity();
					add_component<WorldPosition>(floor_tiling, tile.x, tile.y);
					std::string s(1, syms[index]);
					add_component<Symbol>(floor_tiling, s);
					add_component<Colored>(floor_tiling, HSL(rng->getFloat(180.0f, 200.0f), rng->getFloat(0.0f, 0.5f), 0.8f));
					index = (index + 1) % 4;
				}
			}
		}
	}
}

void WorldCrafting::create_lounge(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();
	
	int min_x = 1000;
	int min_y = 1000;
	int max_x = 0;
	int max_y = 0;

	for (auto tile : tiles)
	{
		if (tile.x < min_x) min_x = tile.x;
		if (tile.y < min_y) min_y = tile.y;
		if (tile.x > max_x) max_x = tile.x;
		if (tile.y > max_y) max_y = tile.y;

		level.dig[tile.x][tile.y] = '.';
		level.hues[tile.x][tile.y] = rng->getFloat(0.0f, 6.0f) * 60.0f + rng->getFloat(-10.0f, 10.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.4f, 0.75f);
	}

	for (int i = min_x; i <= max_x; i++)
	{
		for (int j = min_y; j <= max_y; j++)
		{
			int dx = 0;
			int dy = 0;

			if (rng->getInt(0, 100) > 75)
			{
				dx = rng->getInt(-1, 1);
				dy = rng->getInt(-1, 1);
			}

			if ((i + j) % 2 == 0) continue;
			auto tile = WorldPosition{ i + dx, j + dy };
			
			if (std::find(tiles.begin(), tiles.end(), tile) != tiles.end())
			{
				auto carpet = create_entity();
				add_component<WorldPosition>(carpet, tile.x, tile.y);
				std::string s(1, rng->getInt(0, 100) > 50 ? 'm' : 'n');
				add_component<Symbol>(carpet, s);
				add_component<Colored>(carpet, HSL(rng->getFloat(0.0f, 10.0f) * 36.0f + rng->getFloat(-20.0f, 20.0f), 
					rng->getFloat(0.25f, 0.45f), 0.7f));

				if (dx != 0 || dy != 0)
				{
					auto sell = create_entity();
					add_component<WorldPosition>(sell, i, j);
					std::string s(1, rng->getInt(0, 100) > 50 ? 'a' - 1 : 'a');
					add_component<Symbol>(sell, s);
					add_component<Colored>(sell, HSL(rng->getFloat(0.0f, 10.0f) * 36.0f + rng->getFloat(-20.0f, 20.0f),
						rng->getFloat(0.5f, 1.0f), 0.8f));
				}
			}
		}
	}
}

void WorldCrafting::create_drinking_hole(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_shrine(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	char syms[4]{ 'i', 'j', 'k', 'l' };
	TCODRandom* rng = TCODRandom::getInstance();

	int min_x = 1000;
	int min_y = 1000;
	int max_x = 0;
	int max_y = 0;

	for (auto tile : tiles)
	{
		level.hues[tile.x][tile.y] = rng->getFloat(0.0f, 360.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.0f, 0.3f);
		level.vals[tile.x][tile.y] = 1.0f;

		if (tile.x < min_x) min_x = tile.x;
		if (tile.y < min_y) min_y = tile.y;
		if (tile.x > max_x) max_x = tile.x;
		if (tile.y > max_y) max_y = tile.y;
	}

	int center_x = (min_x + max_x) / 2;
	int center_y = (min_y + max_y) / 2;
	center = WorldPosition{ center_x, center_y };
	auto max_distance = center.distance(WorldPosition{ min_x, max_x });

	for (int i = min_x; i <= max_x; i++)
	{
		for (int j = min_y; j <= max_y; j++)
		{
			if (i % 2 == 0 && (i + j) % 2 == 0)
			{
				auto tile = WorldPosition{ i, j };
				if (std::find(tiles.begin(), tiles.end(), tile) != tiles.end())
				{
					if (rng->getInt(0, 100) > 90) continue;

					auto floor_tiling = create_entity();
					add_component<WorldPosition>(floor_tiling, tile.x, tile.y);
					std::string s(1, syms[rng->getInt(0, 3)]);
					add_component<Symbol>(floor_tiling, s);
					add_component<Colored>(floor_tiling, HSL(rng->getFloat(180.0f, 200.0f), rng->getFloat(0.0f, 0.5f), 0.5f));
				}
			}
		}
	}
}

void WorldCrafting::create_abandoned_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{

	TCODRandom* rng = TCODRandom::getInstance();

	static constexpr int sym_size = 2;
	static char syms[sym_size]{ WIRE_SYM, CHEST_SYM };

	for (auto tile : tiles)
	{
		if (rng->getInt(0, 100) > 90)
			level.dig[tile.x][tile.y] = rng->getInt('v', 'y');
		else
			level.dig[tile.x][tile.y] = rng->getInt(0, 100) >= 50 ? FLOOR_SYM : '.';

		level.hues[tile.x][tile.y] = rng->getFloat(-5.0f, 75.0f);
		level.sats[tile.x][tile.y] = rng->getFloat(0.4f, 0.7f);
	}

	shuffle(tiles);

	for (int i = 0; i < tiles.size() / 4; i++)
	{
		auto tile = tiles.back();
		tiles.pop_back();

		auto ware = create_wares(tile, 'Z' + rng->getInt(1, 12));
		add_component<Colored>(ware, HSL(rng->getFloat(0.0f, 360.0f), rng->getFloat(0.15f, 0.35f), 1.0f));
		remove_component<Blocked>(ware);
		add_tag_component<Exhausting>(ware);
	}
}

void WorldCrafting::create_black_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_monolith(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	int i = rng->getInt(0, tiles.size() - 1);
	center = tiles[i];
	std::sort(tiles.begin(), tiles.end(), [&](WorldPosition& a, WorldPosition& b) { return a.distance(center) < b.distance(center); });

	int c = 0;
	for (auto tile : tiles)
	{
		level.hues[tile.x][tile.y] = rng->getFloat(-20.0f, 20.0f);
		level.sats[tile.x][tile.y] = tile.distance(center) / 10.0f;
		level.vals[tile.x][tile.y] = rng->getFloat(0.95f, 1.0f);

		if (i == c)
		{
			auto entity = create_entity();
			std::string s(1, MONOLITH_SYM);
			add_component<Symbol>(entity, s);
			auto& wp = add_component<WorldPosition>(entity);
			wp.x = center.x;
			wp.y = center.y;

			block_sight_walking(entity, wp);
			add_component<Weight>(entity, (int)MONOLITH_SYM * 5);

			add_tag_component<Shimmering>(entity);

			add_component<Colored>(entity, HSL(0.0f, rng->getFloat(0.85f, 1.0f), rng->getFloat(0.5f, 0.85f)));
			add_component<BumpDefault>(entity, CommandType::Inspect, Command{});

			add_component<PsychicEffect>(entity, (PsychicEffectKind)rng->getInt(0, (int)PsychicEffectKind::COUNT), 5);
		}

		c++;
	}
}

void WorldCrafting::create_hidden_nook(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_haunted_spot(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}
