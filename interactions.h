#pragma once

#include "common.h"

std::vector<Entity> tell_buddy_about_event(PeopleMapping& mapping, Entity who, Entity what_event, int time, int event_id, int max_count = 100)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto all_visiting = mapping.get_all_visiting_with(who);
	shuffle(all_visiting);

	std::vector<Entity> buddies_told;

	std::vector<Entity> talk_events[REGION_COUNT];
	if (!all_visiting.empty())
	{
		for (int i = 0; i < std::min(max_count, rng->getInt(1, all_visiting.size())); i++)
		{
			auto person = std::get<0>(all_visiting[i]);
			auto place = std::get<1>(all_visiting[i]);
			talk_events[mapping.graph->get_tag<Place>(place).place_id].push_back(person);
		}
	}

	for (int i = 0; i < REGION_COUNT; i++)
	{
		if (talk_events[i].empty()) continue;

		auto talk_time = time + rng->getInt(1, time / 2);
		if (talk_time > 0) talk_time = -1;

		auto talk_event = mapping.create_event("TALK", talk_time, event_id);				
		mapping.connect(mapping.places[i], talk_event, "FACILITATED", event_id);

		for (auto buddy : talk_events[i])
		{
			mapping.connect(talk_event, buddy, "listened", event_id);
			buddies_told.push_back(buddy);
		}

		mapping.connect(talk_event, what_event, "about", event_id);
	}

	return buddies_told;
}

std::vector<Entity> tell_work_buddy_about_event(PeopleMapping& mapping, Entity who, Entity what_event, int time, int event_id, int max_count = 100)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto all_workers = mapping.get_all_working_with(who);
	shuffle(all_workers);

	auto talk_event = mapping.graph->create_node();
	mapping.graph->label_node(talk_event, "TALK AT WORK");
	
	auto talk_time = time + rng->getInt(1, time / 2);
	if (talk_time > 0) talk_time = -1;
	mapping.graph->tag_node<Time>(talk_event, talk_time);

	std::vector<Entity> buddies_told;

	mapping.graph->label_edge(mapping.graph->create_arrow(who, talk_event), "talked");
	if (!all_workers.empty())
	{
		for (int i = 0; i < std::min(max_count, rng->getInt(1, all_workers.size())); i++)
		{
			auto person = std::get<0>(all_workers[i]);
			auto place = std::get<1>(all_workers[i]);
			mapping.graph->label_edge(mapping.graph->create_arrow(person, talk_event), "listened");
			buddies_told.push_back(person);
		}

		mapping.graph->label_edge(mapping.graph->create_arrow(talk_event, what_event), "about");
	}

	return buddies_told;

}

void murder(PeopleMapping& mapping, int event_id, Entity killer)
{
	auto murder = mapping.graph->create_arrow(killer, mapping.get_victim());
	mapping.graph->label_edge(murder, "murder");
	mapping.graph->tag_edge<CaseEvent>(murder, event_id);
	mapping.killer = killer;
}

void planned_murder(PeopleMapping& mapping, int event_id, Entity killer)
{
	auto murderlike = mapping.graph->create_arrow(killer, mapping.get_victim());
	mapping.graph->label_edge(murderlike, "threaten");
	mapping.graph->tag_edge<CaseEvent>(murderlike, event_id);
}

void murder_debt_scare(PeopleMapping& mapping, int event_id, Entity victim = entt::null, bool is_murder = true)
{
	TCODRandom* rng = TCODRandom::getInstance();

	// some past debt
	auto past_debt_event = mapping.create_event("PAST DEBT", -100, event_id);

	auto active_debt_time = rng->getInt(-30, -7);
	auto active_debt = mapping.create_event("DEBT", active_debt_time, event_id);
	
	auto people = mapping.get_all_people_shuffled();
	auto old_debtee = people.back();
	people.pop_back();

	Entity rich_killer;
	if (mapping.killer != entt::null)
	{
		rich_killer = mapping.killer;
	}
	else
	{
		rich_killer = people.back();
		people.pop_back();
	}

	if (victim == entt::null)
		victim = mapping.get_victim();

	if (is_murder)
		murder(mapping, event_id, rich_killer);
	else
	{
		planned_murder(mapping, event_id, rich_killer);
		// todo: add witness that debt was returned (potentially create a crook)
	}

	mapping.connect(victim, active_debt, "borrowed", event_id);
	mapping.connect(rich_killer, active_debt, "lent", event_id);

	mapping.connect(old_debtee, past_debt_event, "borrowed", event_id);
	mapping.connect(rich_killer, past_debt_event, "threatened", event_id);

	auto economic_stability = mapping.create_topic("STABILITY", event_id);
	mapping.connect(economic_stability, victim, "who", event_id);

	tell_work_buddy_about_event(mapping, victim, economic_stability, event_id, rng->getInt(active_debt_time, -1));
	tell_buddy_about_event(mapping, victim, economic_stability, event_id, rng->getInt(active_debt_time, -1));

	auto scuffle_time = rng->getInt(-3, -1);
	auto short_scuffle = mapping.create_event("SCUFFLE", scuffle_time, event_id);
	
	mapping.connect(short_scuffle, active_debt, "about", event_id);
	mapping.connect(rich_killer, short_scuffle, "threatened", event_id);
	mapping.connect(victim, short_scuffle, "was threatened", event_id);
	mapping.connect(old_debtee, short_scuffle, "seen", event_id);

	tell_buddy_about_event(mapping, old_debtee, short_scuffle, scuffle_time, event_id, 1);
}

void murder_old_grievance_revenge(PeopleMapping& mapping, int event_id, Entity victim = entt::null, bool is_murder = true)
{
	TCODRandom* rng = TCODRandom::getInstance();

	auto people = mapping.get_all_people_shuffled();
	Entity killer;
	if (mapping.killer != entt::null && rng->getInt(0, 100) > 65)
	{
		killer = mapping.killer;
	}
	else
	{
		killer = people.back();
		people.pop_back();
	}

	if (victim == entt::null)
		victim = mapping.get_victim();

	if (is_murder)
		murder(mapping, event_id, killer);
	else
	{
		planned_murder(mapping, event_id, killer);
		// todo: add witness that grievance was settled (event that someone saw)
	}

	// some past event, war, love, whatever
	auto past_event = mapping.create_event("HARSH PAST", -100, event_id);
	
	// victim has a past
	mapping.connect(victim, past_event, "bullied", event_id);
	mapping.connect(killer, victim, "afraid", event_id);
	mapping.connect(killer, past_event, "was bullied", event_id);

	auto buddies = tell_buddy_about_event(mapping, killer, past_event, -100, event_id, 2);
	for (auto buddy : buddies)
	{
		if (rng->getInt(0, 100) > 90)
			mapping.connect(buddy, killer, "sympathy", event_id);
		else if (rng->getInt(0, 100) > 70)
			mapping.connect(buddy, victim, "afraid", event_id);
	}

	// some recent violent event
	auto recent_violent_time = rng->getInt(-14, -7);
	auto recent_violent_event = mapping.create_event("SCUFFLE", recent_violent_time, event_id);

	auto places = mapping.get_all_places_shuffled();
	auto place = places.back(); places.pop_back();
	
	mapping.connect(recent_violent_event, place, "NEAR", event_id);

	auto all_similar_visit = mapping.get_all_visiting_with(killer);
	Entity almost_victim = entt::null;

	if (all_similar_visit.size() > 0)
	{
		shuffle(all_similar_visit);
		auto v = all_similar_visit.back(); all_similar_visit.pop_back();

		almost_victim = std::get<0>(v);
	}
	else
	{
		almost_victim = people.back(); people.pop_back();
	}

	mapping.connect(almost_victim, recent_violent_event, "was attacked", event_id);
	mapping.connect(killer, recent_violent_event, "mistakenly attacked", event_id);
	mapping.connect(almost_victim, killer, "afraid", event_id);

	tell_work_buddy_about_event(mapping, almost_victim, recent_violent_event, recent_violent_time, event_id);
	for (auto buddy : buddies)
	{
		if (rng->getInt(0, 100) > 75)
			mapping.connect(buddy, killer, "afraid", event_id);		
	}

	tell_buddy_about_event(mapping, almost_victim, recent_violent_event, recent_violent_time, event_id, 2);
	for (auto buddy : buddies)
	{
		if (rng->getInt(0, 100) > 50)
			mapping.connect(buddy, killer, "afraid", event_id);
	}

	auto lookalike = mapping.create_topic("SIMILAR", event_id);
	mapping.connect(almost_victim, lookalike, "similar to", event_id);
	mapping.connect(victim, lookalike, "similar to", event_id);
	mapping.connect(almost_victim, victim, "looks like", event_id);
	mapping.connect(recent_violent_event, lookalike, "reason", event_id);

	auto all_victims_people = mapping.get_all_related_with(victim);
	std::sort(all_victims_people.begin(), all_victims_people.end());

	auto all_almost_victims_people = mapping.get_all_related_with(almost_victim);
	std::sort(all_almost_victims_people.begin(), all_almost_victims_people.end());

	std::vector<std::tuple<PersonEntity, PlaceEntity>> intersection;
	
	std::set_intersection(
		all_victims_people.begin(), all_victims_people.end(), 
		all_almost_victims_people.begin(), all_almost_victims_people.end(),
		std::back_inserter(intersection));

	int count = 0;
	for (auto person_who_knows_both : intersection)
	{
		auto person = std::get<0>(person_who_knows_both);
		mapping.connect(person, lookalike, "knows", event_id);
		count++;
		if (count > 3) break;
	}
}
