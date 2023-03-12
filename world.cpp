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

void WorldCrafting::create_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{
	TCODRandom* rng = TCODRandom::getInstance();

	static constexpr int sym_size = 2;
	static char syms[sym_size]{ WIRE_SYM, CHEST_SYM };

	std::sort(tiles.begin(), tiles.end(), [&](WorldPosition& a, WorldPosition& b) { return a.distance(center) > b.distance(center); });

	for (int i = 0; i < tiles.size() / 3; i++)
	{
		if (rng->getInt(0, 100) > 50) continue;

		auto tile = tiles.back();
		tiles.pop_back();

		auto entity = create_entity();
		auto c = syms[rng->getInt(0, sym_size - 1)];
		std::string s(1, c);
		add_component<Symbol>(entity, s);
		auto& wp = add_component<WorldPosition>(entity);
		wp.x = tile.x;
		wp.y = tile.y;

		add_tag_component<Blocked>(entity);
		add_component<Weight>(entity, (int)c * 5);

		if (c == CHEST_SYM)
		{
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
		}
		else if (c == WIRE_SYM)
		{
			add_component<Colored>(entity, HSL(rng->getFloat(-180.0f, -160.0f), rng->getFloat(0.5f, 0.75f), rng->getFloat(0.5f, 0.85f)));
			add_component<BumpDefault>(entity, CommandType::Inspect, Command{});
		}
	}
}

void WorldCrafting::create_machine_shop(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_library(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_cleric(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_inn(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_temple(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_mess_hall(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_gymnasium(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_mineshaft(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_junkyard(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_foundry(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_skyport(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_lounge(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_drinking_hole(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_shrine(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_abandoned_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_black_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_monolith(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_hidden_nook(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}

void WorldCrafting::create_haunted_spot(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center)
{}
